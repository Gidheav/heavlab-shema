import re

file_path = r'C:\Users\DELL\Desktop\Apps\HV-Bible\hv-desktop\src\shell\ribbon.rs'
with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

content = re.sub(r'const TAB_H: f32 = 21\.0;\n', '', content)
content = re.sub(r'const BODY_H: f32 = 48\.0;\n', '', content)

show_old = '''pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    let sid = egui::Id::new("hvb_ribbon_v3");
    let mut state: RibbonState = ctx.data_mut(|d| d.get_temp(sid).unwrap_or_default());'''

show_new = '''pub fn show(ctx: &Context, app: &mut HvBibleApp) {
    let sid = egui::Id::new("hvb_ribbon_v3");
    let mut state: RibbonState = ctx.data_mut(|d| d.get_temp(sid).unwrap_or_default());
    let base_px = app.current_theme.font_config.size.to_pixels();
    let tab_h = (base_px * 1.5).max(21.0).round();
    let body_h = (base_px * 3.5).max(48.0).round();'''

content = content.replace(show_old, show_new)
content = content.replace('.exact_height(TAB_H + BODY_H)', '.exact_height(tab_h + body_h)')
content = content.replace('egui::vec2(full.width(), TAB_H)', 'egui::vec2(full.width(), tab_h)')
content = content.replace('full.min + egui::vec2(0.0, TAB_H)', 'full.min + egui::vec2(0.0, tab_h)')
content = content.replace('egui::vec2(full.width(), BODY_H)', 'egui::vec2(full.width(), body_h)')
content = content.replace('ui.set_height(BODY_H);', 'ui.set_height(body_h);')

content = content.replace('if tab_btn(ui, tab, state.active_tab == i) {', 'if tab_btn(ui, tab, state.active_tab == i, tab_h) {')
content = content.replace('fn tab_btn(ui: &mut egui::Ui, label: &str, active: bool) -> bool {', 'fn tab_btn(ui: &mut egui::Ui, label: &str, active: bool, tab_h: f32) -> bool {')
content = content.replace('egui::vec2(0.0, TAB_H)', 'egui::vec2(0.0, tab_h)')

content = content.replace('if ghost_icon(ui, "^").clicked() {', 'if ghost_icon(ui, "^", tab_h).clicked() {')
content = content.replace('fn ghost_icon(ui: &mut egui::Ui, icon: &str) -> egui::Response {', 'fn ghost_icon(ui: &mut egui::Ui, icon: &str, tab_h: f32) -> egui::Response {')
content = content.replace('egui::vec2(22.0, TAB_H)', 'egui::vec2(22.0, tab_h)')

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
