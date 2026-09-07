# Vana Haven

A local desktop app for tracking Final Fantasy XI character progression — jobs,
gear-set completion, key item cooldowns, daily activities, currencies, and
Limbus progress — fed live by an in-game Windower addon.

## Development

- App: `npm install`, then `npm run tauri dev`
- Addon: copy `addon/VanaHaven` into your `Windower4/addons/` folder, then
  `//lua l vanahaven` in-game.

See `docs/superpowers/specs/` for the design spec (local reference only, not
tracked in git).
