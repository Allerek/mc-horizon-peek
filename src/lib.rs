use mod_api::{
    BlockPos, BlockState, ClientHost, CrosshairTarget, GuiRenderPostEvent, Hooks, Identifier,
    ItemKey,
};

const PANEL_TOP: f32 = 4.0;
const PADDING: f32 = 5.0;
const ICON_SIZE: f32 = 18.0;
const LINE_HEIGHT: f32 = 10.0;
const BACKGROUND: u32 = 0xe0181420;
const BORDER: u32 = 0xff5e159a;
const INNER_BORDER: u32 = 0xff32104f;
const TITLE: u32 = 0xffffffff;
const DETAIL: u32 = 0xffc8c8c8;
const SOURCE: u32 = 0xff6060ff;
const HEART: u32 = 0xffff3b43;

#[derive(Default)]
struct WhatAmILookingAt;

struct Panel {
    title: String,
    details: Vec<(String, u32)>,
    source: String,
    icon: Option<ItemKey>,
}

#[mod_api::horizon_mod]
impl Hooks for WhatAmILookingAt {
    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        if event.gui_hidden || !host.playing() {
            return;
        }

        let Some(target) = host.crosshair_target() else {
            return;
        };
        let panel = match target {
            CrosshairTarget::Block { pos, state, .. } => block_panel(host, pos, state),
            CrosshairTarget::Entity { entity, .. } => {
                let id = entity
                    .kind(host)
                    .map(|kind| kind.id(host))
                    .unwrap_or_else(|| "minecraft:entity".to_owned());
                let title = entity.name(host).unwrap_or_else(|| human_name(&id));
                let mut details = Vec::new();

                if let Some(living) = host.living(entity)
                    && let (Some(health), Some(max_health)) =
                        (living.health(host), living.max_health(host))
                {
                    details.push((health_line(health, max_health), HEART));
                }

                Panel {
                    title,
                    details,
                    source: namespace_name(&id),
                    icon: None,
                }
            }
            _ => return,
        };

        draw_panel(event, &panel);
    }
}

fn block_panel(host: &dyn ClientHost, _pos: BlockPos, state: BlockState) -> Panel {
    let serialized = host
        .block_state_string(state)
        .unwrap_or_else(|| "minecraft:unknown".to_owned());
    let (id, properties) = split_block_state(&serialized);
    let mut details = Vec::new();

    if let Some(tool) = preferred_tool(id) {
        details.push((format!("Correct Tool: {tool}"), DETAIL));
    }
    if let Some(properties) = properties {
        details.push((properties.replace(',', ", "), DETAIL));
    }

    let icon = Identifier::parse(id)
        .ok()
        .and_then(|identifier| host.item_key(&identifier));

    Panel {
        title: human_name(id),
        details,
        source: namespace_name(id),
        icon,
    }
}

fn split_block_state(state: &str) -> (&str, Option<&str>) {
    let Some(start) = state.find('[') else {
        return (state, None);
    };
    let properties = state
        .get(start + 1..)
        .and_then(|properties| properties.strip_suffix(']'))
        .filter(|properties| !properties.is_empty());
    (&state[..start], properties)
}

fn preferred_tool(id: &str) -> Option<&'static str> {
    let path = id.rsplit(':').next().unwrap_or(id);
    if path.contains("ore")
        || path.contains("stone")
        || path.contains("deepslate")
        || path.contains("brick")
        || path.contains("anvil")
        || path.contains("furnace")
    {
        Some("Pickaxe")
    } else if path.contains("log")
        || path.contains("wood")
        || path.contains("stem")
        || path.contains("hyphae")
        || path.contains("planks")
    {
        Some("Axe")
    } else if path.contains("dirt")
        || path.contains("sand")
        || path.contains("gravel")
        || path.contains("clay")
        || path.contains("snow")
    {
        Some("Shovel")
    } else if path.contains("leaves") || path.contains("wool") || path.contains("web") {
        Some("Shears")
    } else {
        None
    }
}

fn health_line(health: f32, max_health: f32) -> String {
    let filled = (health.max(0.0) / 2.0).ceil() as usize;
    let total = (max_health.max(0.0) / 2.0).ceil() as usize;
    let mut line = "♥".repeat(filled.min(total));
    line.push_str(&"♡".repeat(total.saturating_sub(filled)));
    format!("{line}  {health:.0}/{max_health:.0}")
}

fn namespace_name(id: &str) -> String {
    let namespace = id
        .split_once(':')
        .map(|(namespace, _)| namespace)
        .unwrap_or("minecraft");
    human_name(namespace)
}

fn human_name(id: &str) -> String {
    let path = id.rsplit(':').next().unwrap_or(id);
    path.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn draw_panel(event: &mut GuiRenderPostEvent<'_>, panel: &Panel) {
    let [screen_width, _] = event.batch.size();
    let text_left = PADDING + if panel.icon.is_some() { ICON_SIZE + PADDING } else { 0.0 };
    let mut content_width = event.batch.text_width(&panel.title);
    for (line, _) in &panel.details {
        content_width = content_width.max(event.batch.text_width(line));
    }
    content_width = content_width.max(event.batch.text_width(&panel.source));

    let line_count = 2 + panel.details.len();
    let panel_width = (text_left + content_width + PADDING).max(92.0);
    let panel_height = PADDING * 2.0 + line_count as f32 * LINE_HEIGHT;
    let left = ((screen_width - panel_width) * 0.5).floor();

    event.batch.fill(
        [left - 2.0, PANEL_TOP - 2.0, panel_width + 4.0, panel_height + 4.0],
        BORDER,
    );
    event.batch.fill(
        [left - 1.0, PANEL_TOP - 1.0, panel_width + 2.0, panel_height + 2.0],
        INNER_BORDER,
    );
    event
        .batch
        .fill([left, PANEL_TOP, panel_width, panel_height], BACKGROUND);

    if let Some(icon) = panel.icon {
        event
            .batch
            .item(icon, [left + PADDING, PANEL_TOP + PADDING], ICON_SIZE);
    }

    let x = left + text_left;
    let mut y = PANEL_TOP + PADDING;
    event.batch.text(&panel.title, [x, y], TITLE, true);
    y += LINE_HEIGHT;
    for (line, color) in &panel.details {
        event.batch.text(line, [x, y], *color, true);
        y += LINE_HEIGHT;
    }
    event.batch.text(&panel.source, [x, y], SOURCE, true);
}
