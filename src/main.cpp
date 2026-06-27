// NightZoom FPS Limiter - a ReShade addon that hard-caps the frame rate to 60 FPS.
//
// Build target: single DLL renamed to NightZoom.addon64 (Windows x64).
// Requires the ADDON-ENABLED build of ReShade.
//
// How it works:
//  - reshade::addon_event::present fires once per frame. We measure the time since
//    the previous present and, when the limiter is enabled, block until exactly one
//    60 FPS frame interval has elapsed using a hybrid sleep + busy-wait.
//  - The overlay callback draws a dedicated "NightZoom FPS Limiter" window.
//  - The checkbox state is persisted via ReShade's own config (no custom file).

#include <imgui.h>          // Must be included BEFORE reshade.hpp so the overlay wrappers compile.
#include <reshade.hpp>

#include <Windows.h>
#include <shellapi.h>       // ShellExecuteA (open Discord link)
#include <intrin.h>         // _mm_pause (spin-wait hint)
#include <chrono>
#include <thread>
#include <atomic>

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Hardcoded cap. Exactly 60.000 FPS -> one frame every 1/60 second.
static constexpr double kTargetFps = 60.0;
static constexpr std::chrono::duration<double> kFrameInterval{ 1.0 / kTargetFps };

static constexpr const char *kConfigSection = "NightZoom";
static constexpr const char *kConfigKey     = "LimitTo60";

static constexpr const char *kDiscordUrl = "https://discord.gg/nightzoom";

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

// Read in the present callback every frame; toggled in the overlay callback.
static std::atomic<bool> g_limit_enabled{ false };

using clock_type = std::chrono::high_resolution_clock;
static clock_type::time_point g_last_present = clock_type::now();

// ---------------------------------------------------------------------------
// Frame pacing
// ---------------------------------------------------------------------------

// Hybrid sleep + busy-wait. A plain Sleep() stutters because of Windows timer
// granularity, so we sleep until ~1 ms before the target and spin the remainder.
static void on_present(reshade::api::command_queue *, reshade::api::swapchain *,
                       const reshade::api::rect *, const reshade::api::rect *,
                       uint32_t, const reshade::api::rect *)
{
	if (!g_limit_enabled.load(std::memory_order_relaxed))
	{
		// No cap: just keep the timestamp fresh so re-enabling does not over-sleep.
		g_last_present = clock_type::now();
		return;
	}

	const clock_type::time_point target = g_last_present + std::chrono::duration_cast<clock_type::duration>(kFrameInterval);

	// Coarse phase: sleep until ~1 ms before the target (timeBeginPeriod(1) keeps this tight).
	for (;;)
	{
		const clock_type::time_point now = clock_type::now();
		if (now >= target)
			break;

		const auto remaining = target - now;
		if (remaining > std::chrono::milliseconds(1))
			std::this_thread::sleep_for(remaining - std::chrono::milliseconds(1));
		else
			break; // Hand off to the spin phase for the last sub-millisecond.
	}

	// Fine phase: busy-wait the remainder for frame-accurate pacing.
	while (clock_type::now() < target)
		_mm_pause();

	g_last_present = clock_type::now();
}

// ---------------------------------------------------------------------------
// Config persistence (ReShade config, not a custom file)
// ---------------------------------------------------------------------------

static void on_init_effect_runtime(reshade::api::effect_runtime *runtime)
{
	bool value = false;
	if (reshade::get_config_value(runtime, kConfigSection, kConfigKey, value))
		g_limit_enabled.store(value, std::memory_order_relaxed);
}

// ---------------------------------------------------------------------------
// Logo (one clearly-commented function, trivial to swap for a real texture)
// ---------------------------------------------------------------------------

// Draws the logo at the top of the window. Currently a bordered text placeholder
// at a fixed 200x80 size.
//
// TODO: To show a real logo, read "NightZoom_logo.png" from the addon's own
// directory, create a texture on the runtime's device (runtime->get_device() ->
// create_resource / create_resource_view), and pass its handle to ImGui::Image()
// here instead of the bordered box below.
static void draw_logo()
{
	const ImVec2 size(200.0f, 80.0f);
	const ImVec2 pos = ImGui::GetCursorScreenPos();

	ImDrawList *draw = ImGui::GetWindowDrawList();
	draw->AddRect(pos, ImVec2(pos.x + size.x, pos.y + size.y),
	              ImGui::GetColorU32(ImGuiCol_Border), 4.0f);

	const char *label = "[ NightZoom logo ]";
	const ImVec2 text_size = ImGui::CalcTextSize(label);
	draw->AddText(ImVec2(pos.x + (size.x - text_size.x) * 0.5f,
	                     pos.y + (size.y - text_size.y) * 0.5f),
	              ImGui::GetColorU32(ImGuiCol_Text), label);

	ImGui::Dummy(size); // Reserve the layout space the box occupies.
}

// ---------------------------------------------------------------------------
// Overlay window
// ---------------------------------------------------------------------------

static void draw_overlay(reshade::api::effect_runtime *runtime)
{
	draw_logo();
	ImGui::Spacing();

	bool enabled = g_limit_enabled.load(std::memory_order_relaxed);
	if (ImGui::Checkbox("Limit to 60 FPS", &enabled))
	{
		g_limit_enabled.store(enabled, std::memory_order_relaxed);
		g_last_present = clock_type::now(); // Reset pacing baseline on toggle.
		reshade::set_config_value(runtime, kConfigSection, kConfigKey, enabled);
	}

	ImGui::Spacing();
	ImGui::Separator();
	ImGui::TextUnformatted("Made by Nipeno");

	// Discord link: button opens the invite; full URL shown as a selectable fallback.
	if (ImGui::Button("Join the Discord"))
		ShellExecuteA(nullptr, "open", kDiscordUrl, nullptr, nullptr, SW_SHOWNORMAL);
	ImGui::SameLine();
	ImGui::TextUnformatted(kDiscordUrl); // Selectable/copyable if the click is blocked.
}

// ---------------------------------------------------------------------------
// Addon metadata (read by ReShade)
// ---------------------------------------------------------------------------

extern "C" __declspec(dllexport) const char *NAME = "NightZoom FPS Limiter";
extern "C" __declspec(dllexport) const char *DESCRIPTION =
	"Hard-caps the game's frame rate to exactly 60 FPS. Made by Nipeno.";

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID)
{
	switch (reason)
	{
	case DLL_PROCESS_ATTACH:
		if (!reshade::register_addon(hModule))
			return FALSE;
		timeBeginPeriod(1); // Tighten sleep granularity for the pacing loop.
		reshade::register_event<reshade::addon_event::init_effect_runtime>(on_init_effect_runtime);
		reshade::register_event<reshade::addon_event::present>(on_present);
		reshade::register_overlay("NightZoom FPS Limiter", draw_overlay);
		break;
	case DLL_PROCESS_DETACH:
		reshade::unregister_addon(hModule);
		timeEndPeriod(1);
		break;
	}
	return TRUE;
}
