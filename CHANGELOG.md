## Upcoming

  Nothing as of now

# v0.2.1 (2024-02-03)

- Now using ordr's new websocket url
- Enforcing stricter coding standards

## v0.2.0 (2023-09-06)

- Breaking changes:
  - Removed `Skin::has_cursor_middle` and `Render::has_cursor_middle` ([#2])
  - Added the fields `RenderOptions::show_strain_graph`, `RenderOptions::show_slider_breaks`, and `RenderOptions::ignore_fail`
  - Removed `PartialEq` and `Eq` impls for `RenderList` and `Render`
  - The field `Render::skin` is now of type `RenderSkinOption<'static>`
  - Added the field `Render::options`

- Additions:
  - Implement `Default` for `OrdrClient`

## v0.1.0 (2023-07-20)

[#2]: https://github.com/MaxOhn/rosu-render/pull/2