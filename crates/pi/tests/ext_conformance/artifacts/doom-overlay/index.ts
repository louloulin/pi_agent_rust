/**
 * DOOM Overlay Demo - Play DOOM as an overlay
 *
 * Usage: pi --extension ./examples/extensions/doom-overlay
 *
 * Commands:
 *   /doom-overlay - Play DOOM in an overlay (Q to pause/exit)
 *
 * This demonstrates that overlays can handle real-time game rendering at 35 FPS.
 */

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { DoomOverlayComponent } from "./doom-component.js";
import { DoomEngine } from "./doom-engine.js";
import { ensureWadFile } from "./wad-finder.js";

async function yieldToUi(): Promise<void> {
	await new Promise((resolve) => setTimeout(resolve, 0));
}

// Persistent engine instance - survives between invocations
let activeEngine: DoomEngine | null = null;
let activeWadPath: string | null = null;

export default function (pi: ExtensionAPI) {
	pi.registerCommand("doom-overlay", {
		description: "Play DOOM as an overlay. Q to pause and exit.",

		handler: async (args, ctx) => {
			if (!ctx.hasUI) {
				ctx.ui.notify("DOOM requires interactive mode", "error");
				return;
			}

			const wad = args?.trim()
				? args.trim()
				: await ensureWadFile((message) => ctx.ui.notify(message, "info"));

			if (!wad) {
				ctx.ui.notify("Failed to download DOOM WAD file. Check your internet connection.", "error");
				return;
			}

			try {
				let isResume = false;
				if (activeEngine && activeWadPath === wad) {
					ctx.ui.notify("Resuming DOOM...", "info");
					await yieldToUi();
					isResume = true;
				} else {
					ctx.ui.notify(`Loading DOOM from ${wad}...`, "info");
					await yieldToUi();
					activeEngine = new DoomEngine(wad, (message) => ctx.ui.notify(message, "info"));
					ctx.ui.notify("Initializing DOOM engine...", "info");
					await yieldToUi();
					await activeEngine.init();
					ctx.ui.notify("DOOM engine initialized.", "info");
					activeWadPath = wad;
				}

				ctx.ui.notify("Opening DOOM overlay...", "info");
				await ctx.ui.custom(
					(tui, _theme, _keybindings, done) =>
						new DoomOverlayComponent(tui, activeEngine!, () => done(undefined), isResume),
					{
						overlay: true,
						overlayOptions: {
							width: "75%",
							maxHeight: "95%",
							anchor: "center",
							margin: { top: 1 },
						},
					},
				);
			} catch (error) {
				ctx.ui.notify(`Failed to load DOOM: ${error}`, "error");
				activeEngine = null;
				activeWadPath = null;
			}
		},
	});
}
