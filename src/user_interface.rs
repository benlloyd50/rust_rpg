use std::collections::hash_map::Entry;
use std::collections::HashMap;

use bracket_rex::prelude::*;
use bracket_terminal::{prelude::{
    render_draw_buffer, BTerm, ColorPair, DrawBatch, Point, TextAlign, BLACK, RGBA,
}, rex::xp_to_draw_batch};
use specs::World;

use crate::CL_TEXT;

pub fn draw_ui(ecs: &World, ctx: &mut BTerm) {
    let ui_layout = ecs.fetch::<UILayout>();
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_TEXT).cls();

    render_ui(&ui_layout, &mut draw_batch);

    draw_batch.submit(CL_TEXT).expect("Batch error??");
    render_draw_buffer(ctx).expect("Render error??");
}

fn render_ui(layout: &UILayout, batch: &mut DrawBatch) {
    for element in layout.components.values() {
        if !element.is_visible {
            continue;
        }
        element.draw_in_batch(batch);
    }
}

/// Takes ui creation requests and creates them onto the ui layout
pub fn layout_ui_components(ecs: &World) {
    let mut ui_requests = ecs.fetch_mut::<UICreationRequests>();
    let mut layout = ecs.fetch_mut::<UILayout>();

    for request in ui_requests.requests.iter() {
        let mut parent_pos = Point::new(0, 0);

        if let Some(parent) = &request.parent {
            parent_pos = layout.components[parent].absolute_pos;
            match layout.components.entry(parent.to_string()) {
                Entry::Occupied(mut o) => {
                    o.get_mut().children.push(request.identifier.clone());
                }
                Entry::Vacant(_) => {
                    panic!(
                        "Can not create child: {} because parent: {} does not exsit!",
                        request.identifier, parent
                    )
                }
            }
        }

        parent_pos = calc_absolute_point(&parent_pos, &request, &layout);

        layout.components.insert(
            request.identifier.clone(),
            UIComponent {
                absolute_pos: parent_pos,
                text: request.text.clone(),
                z_priority: request.z_priority,
                children: vec![],
                is_visible: true,
                component_type: request.component_type.clone(),
            },
        );
    }

    ui_requests.requests.clear();
}

fn calc_absolute_point(
    parent_pos: &Point,
    request: &UIComponentRequest,
    layout: &UILayout,
) -> Point {
    let absolute_x = parent_pos.x
        + (request.relative_pos_percent.0 as f32 / 100.0 * layout.c_width as f32).trunc() as i32;
    let absolute_y = parent_pos.y
        + (request.relative_pos_percent.1 as f32 / 100.0 * layout.c_height as f32).trunc() as i32;

    if absolute_x < 0
        || absolute_x > layout.c_width as i32
        || absolute_y < 0
        || absolute_y > layout.c_height as i32
    {
        panic!(
            "ERROR: Calculation out of bounds. {} with position {} {}",
            request.identifier, absolute_x, absolute_y
        );
    }

    Point::new(absolute_x, absolute_y)
}

#[derive(Default)]
pub struct UILayout {
    c_width: usize,
    c_height: usize,
    components: HashMap<String, UIComponent>,
}

impl UILayout {
    pub fn new(c_width: usize, c_height: usize) -> Self {
        Self {
            c_width,
            c_height,
            components: HashMap::new(),
        }
    }
}

struct UIComponent {
    absolute_pos: Point,
    text: String,
    z_priority: u32,
    children: Vec<String>,
    is_visible: bool,
    component_type: UIComponentType,
}

impl UIComponent {
    #[rustfmt::skip]
    /// Draws the UIComponent onto the batch respective to the `component_type`
    pub fn draw_in_batch(&self, draw_batch: &mut DrawBatch) {
        match &self.component_type {
            UIComponentType::Label { background } => {
                draw_batch.printer_with_z(self.absolute_pos, &self.text, TextAlign::Left, *background, self.z_priority);
            }
            UIComponentType::RexDrawing { file } => {
                xp_to_draw_batch(file, draw_batch, 0, 0);
            }
            _ => {todo!("getting around to implementing printer for each component_type")}
        }
    }
}

pub struct UIComponentRequest {
    /// Something to uniquely identify the component with
    identifier: String,
    /// The text that gets printed to the screen
    text: String,
    /// Relative to the screen or parent depending on if this has a parent
    relative_pos_percent: (usize, usize),
    /// Order in which to be drawn
    z_priority: u32,
    /// The owner of the component for relative screen calculation and teardown
    parent: Option<String>,
    /// The type of component being drawn
    component_type: UIComponentType,
}

#[derive(Default)]
pub struct UICreationRequests {
    requests: Vec<UIComponentRequest>,
}

impl UICreationRequests {
    fn add(&mut self, component: UIComponentRequest) -> &mut Self {
        self.requests.push(component);
        self
    }
}

/// Defines the different types of ui elements
/// `z_priority` is only utilized between ui components not game objects
#[derive(Clone)]
pub enum UIComponentType {
    ProgressBar {
        label_bg: Option<RGBA>,
        bar_color: ColorPair, // fg - active bar color, bg - inactive bar color
        curr: usize,
        total: usize,
    },
    TextChoices {
        background: Option<RGBA>,
        highlight: RGBA,
        choices: Vec<Choice>, // Choices should Buttons
    },
    TextBox {
        buffer: String,
    },
    Box {
        size_percent: (usize, usize), // 1 - 100%, (width, height)
        color: ColorPair,
    },
    Label {
        background: Option<RGBA>,
    },
    RexDrawing {
        file: XpFile,
    },
    Button {
        background: Option<RGBA>,
        is_active: bool,
    },
}

#[derive(Clone)]
struct Choice {
    label: String,
    is_active: bool,
}

impl UIComponentRequest {
    pub fn fps_counter(fps: usize) -> UIComponentRequest {
        UIComponentRequest {
            identifier: "FPS".to_string(),
            relative_pos_percent: (90, 2),
            text: format!("#[pink]FPS: {}", fps),
            z_priority: 1,
            component_type: UIComponentType::Label { background: None },
            parent: None,
        }
    }

    pub fn test_long_word() -> UIComponentRequest {
        UIComponentRequest {
            identifier: "TestLongWord".to_string(),
            relative_pos_percent: (2, 4),
            text: "#[white]This is a really long sentence to test formatting, I love lauren! <3#[]"
                .to_string(),
            z_priority: 1,
            component_type: UIComponentType::Label {
                background: Some(BLACK.into()),
            },
            parent: None,
        }
    }

    pub fn test_rex_image(image_name: &str) -> UIComponentRequest {
        match XpFile::from_resource(format!("../resources/rex/{}.xp", image_name).as_str()).ok() {
            Some(file) => UIComponentRequest {
                identifier: "Map Bar".to_string(),
                text: String::new(),
                relative_pos_percent: (50, 5),
                z_priority: 3,
                parent: None,
                component_type: UIComponentType::RexDrawing { file },
            },
            None => panic!("Error trying to load resource path for xp files"),
        }
    }

    // pub fn nice_box(origin: Point, size_percent: (usize, usize), label: impl ToString, z_priority: u32, color: ColorPair) -> Self {
    //     UIComponent {
    //         label: label.to_string(),
    //         z_priority,
    //         pos: origin,
    //         children: vec![],
    //         component_type: UIComponentType::Box { size_percent, color},
    //     }
    // }
    //
}

pub fn initialize_layout(ui: &mut UICreationRequests) {
    ui.add(UIComponentRequest::fps_counter(0))
        .add(UIComponentRequest::test_long_word())
        .add(UIComponentRequest::test_rex_image("map_bar"));
    // .add(String::from("Menu"), UIComponent::nice_box(Point::new(5, 5), (80, 40), "Menu 1", 1, ColorPair::new(WHITE, BLACK)));
}

// /// For creation of new ui components similar to tile anims
// struct InterfaceRequest;
