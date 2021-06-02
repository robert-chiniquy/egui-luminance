# Egui on Luminance

This is a (currently broken) attempt to paint [Egui](https://github.com/emilk/egui)'s UI using [Luminance](https://github.com/phaazon/luminance-rs) for the browser / under WebGL. I'm pushing it as context for this discussion: https://github.com/emilk/egui/discussions/443

This is attempting to draw the UI:

```rust
        let u = self.build_ui(&mut surface, |ctx| {
            egui::SidePanel::left("❤", 200.).show(&ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("hello");
                });
                ui.separator();
                ui.label(format!("t: {}", t));
            });
        });
```

Currently I think the only thing broken is loading Egui's texture. If this worked, it would be a short step to a real (WebGL-only) Egui+Luminance integration, just some factoring, input-output handling, and common-sense perf fixes (this is rebuilding everything every raf() loop) .

Help is welcome!

# dev

```
$ yarn serve
```

http://localhost:8080
