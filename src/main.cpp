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
#include "logo_data.h"      // Embedded NightZoom_logo.png bytes (g_logo_png / g_logo_png_len)

#include <Windows.h>
#include <shellapi.h>       // ShellExecuteA (open Discord link)
#include <intrin.h>         // _mm_pause (spin-wait hint)
#include <wincodec.h>       // WIC: decode NightZoom_logo.png (system component, no extra dep)
#include <wrl/client.h>     // Microsoft::WRL::ComPtr
#include <chrono>
#include <thread>
#include <atomic>
#include <vector>
#include <string>

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

// Logo texture. Created in init_effect_runtime, freed in destroy_effect_runtime.
// All zero -> no logo loaded; draw_logo() falls back to the bordered placeholder.
static reshade::api::device       *g_logo_device = nullptr;
static reshade::api::resource      g_logo_resource = { 0 };
static reshade::api::resource_view g_logo_view = { 0 };
static uint32_t g_logo_width = 0;
static uint32_t g_logo_height = 0;

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

// ---------------------------------------------------------------------------
// Logo loading (one clearly-commented unit)
// ---------------------------------------------------------------------------

// Decode the embedded PNG (g_logo_png) to tightly-packed 32-bit RGBA using WIC
// (a Windows system component). The image is baked into the DLL, so there is no
// external file to ship or expose. Returns false if decoding fails.
static bool decode_png_rgba(std::vector<uint8_t> &pixels, uint32_t &width, uint32_t &height)
{
	using Microsoft::WRL::ComPtr;

	// COM may already be initialised by the game; tolerate a different threading model.
	const HRESULT co = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
	const bool co_owned = SUCCEEDED(co);

	bool ok = false;
	{
		ComPtr<IWICImagingFactory> factory;
		ComPtr<IWICStream> stream;
		ComPtr<IWICBitmapDecoder> decoder;
		ComPtr<IWICBitmapFrameDecode> frame;
		ComPtr<IWICFormatConverter> converter;

		if (SUCCEEDED(CoCreateInstance(CLSID_WICImagingFactory, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&factory))) &&
		    SUCCEEDED(factory->CreateStream(&stream)) &&
		    SUCCEEDED(stream->InitializeFromMemory(const_cast<BYTE *>(g_logo_png), g_logo_png_len)) &&
		    SUCCEEDED(factory->CreateDecoderFromStream(stream.Get(), nullptr, WICDecodeMetadataCacheOnLoad, &decoder)) &&
		    SUCCEEDED(decoder->GetFrame(0, &frame)) &&
		    SUCCEEDED(factory->CreateFormatConverter(&converter)) &&
		    SUCCEEDED(converter->Initialize(frame.Get(), GUID_WICPixelFormat32bppRGBA, WICBitmapDitherTypeNone, nullptr, 0.0, WICBitmapPaletteTypeCustom)))
		{
			UINT w = 0, h = 0;
			if (SUCCEEDED(converter->GetSize(&w, &h)) && w > 0 && h > 0)
			{
				const UINT stride = w * 4;
				pixels.resize(static_cast<size_t>(stride) * h);
				if (SUCCEEDED(converter->CopyPixels(nullptr, stride, static_cast<UINT>(pixels.size()), pixels.data())))
				{
					width = w;
					height = h;
					ok = true;
				}
			}
		}
	}

	if (co_owned)
		CoUninitialize();
	return ok;
}

// Create the GPU texture for the logo on the runtime's device, if the PNG exists.
static void load_logo_texture(reshade::api::effect_runtime *runtime)
{
	std::vector<uint8_t> pixels;
	uint32_t w = 0, h = 0;
	if (!decode_png_rgba(pixels, w, h))
		return; // Decode failed -> placeholder is drawn instead.

	reshade::api::device *device = runtime->get_device();

	const reshade::api::resource_desc desc(
		w, h, 1, 1, reshade::api::format::r8g8b8a8_unorm, 1,
		reshade::api::memory_heap::gpu_only, reshade::api::resource_usage::shader_resource);

	const reshade::api::subresource_data initial{ pixels.data(), w * 4u, w * h * 4u };

	reshade::api::resource res = { 0 };
	if (!device->create_resource(desc, &initial, reshade::api::resource_usage::shader_resource, &res))
		return;

	reshade::api::resource_view view = { 0 };
	if (!device->create_resource_view(res, reshade::api::resource_usage::shader_resource,
	                                  reshade::api::resource_view_desc(reshade::api::format::r8g8b8a8_unorm), &view))
	{
		device->destroy_resource(res);
		return;
	}

	g_logo_device = device;
	g_logo_resource = res;
	g_logo_view = view;
	g_logo_width = w;
	g_logo_height = h;
}

static void free_logo_texture()
{
	if (g_logo_device != nullptr)
	{
		if (g_logo_view.handle != 0)
			g_logo_device->destroy_resource_view(g_logo_view);
		if (g_logo_resource.handle != 0)
			g_logo_device->destroy_resource(g_logo_resource);
	}
	g_logo_device = nullptr;
	g_logo_resource = { 0 };
	g_logo_view = { 0 };
	g_logo_width = g_logo_height = 0;
}

static void on_init_effect_runtime(reshade::api::effect_runtime *runtime)
{
	bool value = false;
	if (reshade::get_config_value(runtime, kConfigSection, kConfigKey, value))
		g_limit_enabled.store(value, std::memory_order_relaxed);

	load_logo_texture(runtime);
}

static void on_destroy_effect_runtime(reshade::api::effect_runtime *)
{
	free_logo_texture();
}

// ---------------------------------------------------------------------------
// Logo drawing
// ---------------------------------------------------------------------------

// Draws the real logo texture if one was loaded; otherwise a bordered
// "[ NightZoom logo ]" placeholder at a fixed 200x80 size.
static void draw_logo()
{
	if (g_logo_view.handle != 0)
	{
		// Fit the logo to a 200px width, preserving aspect ratio.
		const float target_w = 200.0f;
		const float scale = target_w / static_cast<float>(g_logo_width);
		ImGui::Image(static_cast<ImTextureID>(g_logo_view.handle),
		             ImVec2(target_w, static_cast<float>(g_logo_height) * scale));
		return;
	}

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
		reshade::register_event<reshade::addon_event::destroy_effect_runtime>(on_destroy_effect_runtime);
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
