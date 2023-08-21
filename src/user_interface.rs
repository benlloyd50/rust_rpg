use bracket_rex::prelude::*;
use bracket_terminal::{
    prelude::{ColorPair, DrawBatch, Point, TextAlign, BLACK, RGBA},
    rex::xp_to_draw_batch,
};
use specs::World;

use crate::{CL_TEXT, DISPLAY_WIDTH};

pub fn draw_ui(ecs: &World) {
    let ui_layout = ecs.fetch::<UILayout>();
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(CL_TEXT).cls();

    render_ui(&ui_layout, &mut draw_batch);

    draw_batch.submit(CL_TEXT).expect("Batch error??");
}

fn render_ui(layout: &UILayout, batch: &mut DrawBatch) {
    for element in layout.components.iter() {
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
        println!("Init {:?}: pos: {:?}", request.identifier, request.position);

        layout.checked_add(
            UIComponent {
                identifier: request.identifier.clone(),
                absolute_pos: request.position,
                text: request.text.clone(),
                z_priority: request.z_priority,
                is_visible: true,
                component_type: request.component_type.clone(),
            },
        );
    }

    if ui_requests.requests.len() > 0 {
        layout.sort();
    }

    ui_requests.requests.clear();
}

#[derive(Default)]
pub struct UILayout {
    _c_width: usize,
    _c_height: usize,
    /// Components is a weird requirement because it needs two things
    /// - when i insert a value if one exists with the same name we get rid of it
    /// - i want to keep the collection sorted so i can iter it when drawing, need to sort only
    /// after adding new things
    components: Vec<UIComponent>,
}

impl UILayout {
    pub fn new(c_width: usize, c_height: usize) -> Self {
        Self {
            _c_width: c_width,
            _c_height: c_height,
            components: Vec::new(),
        }
    }

    fn checked_add(&mut self, component: UIComponent) {
        if let Some(index) = self.components.binary_search_by(|v| v.identifier.cmp(&component.identifier)).ok() {
            self.components.remove(index);
        }
        self.components.push(component);
    }

    fn sort(&mut self) {
        self.components.sort_by(|lhs, rhs| lhs.z_priority.cmp(&rhs.z_priority));
        println!("First: {:?}", self.components[0].identifier);
    }

    fn get_mut<'a>(&'a mut self, identifier: &str) -> Option<&'a mut UIComponent> {
        for component in self.components.iter_mut() {
            if component.identifier == identifier {
                return Some(component);
            }
        }
        None
    }

}

struct UIComponent {
    identifier: String,
    absolute_pos: Point,
    text: String,
    z_priority: u32,
    is_visible: bool,
    component_type: UIComponentType,
}

impl UIComponent {
    #[rustfmt::skip]
    /// Draws the UIComponent onto the batch respective to the `component_type`
    pub fn draw_in_batch(&self, draw_batch: &mut DrawBatch) {
        match &self.component_type {
            UIComponentType::Label { background, align } => {
                draw_batch.printer(self.absolute_pos, &self.text, *align, *background);
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
    position: Point,
    /// Order in which to be drawn
    z_priority: u32,
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
#[allow(dead_code)]
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
        // The width and height of the box in percent 1 - 100%, (width, height)
        dimensions_percent: (usize, usize),
        color: ColorPair,
    },
    Label {
        background: Option<RGBA>,
        align: TextAlign,
    },
    RexDrawing {
        file: XpFile,
    },
    Button {
        background: Option<RGBA>,
        is_active: bool,
    },
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Choice {
    label: String,
    is_active: bool,
}

const CLEAR: RGBA = RGBA {r: 0.0, g: 0.0, b: 0.0, a: 0.0};

impl UIComponentRequest {
    pub fn log_message(contents: String, order: usize) -> UIComponentRequest {
        UIComponentRequest {
            identifier: format!("log_{}", order),
            text: format!("#[white]{}: {}#[]", order, contents),
            position: Point::new(2, 47 + order),
            z_priority: 3,
            component_type: UIComponentType::Label {
                background: None,
                align: TextAlign::Left,
            },
        }
    }

    pub fn fps_counter(fps: usize) -> UIComponentRequest {
        UIComponentRequest {
            identifier: "FPS".to_string(),
            position: Point::new(DISPLAY_WIDTH * 2 - 8, 2),
            text: format!("#[pink]FPS: {}", fps),
            z_priority: 1,
            component_type: UIComponentType::Label {
                background: Some(CLEAR),
                align: TextAlign::Left,
            },
        }
    }

    #[allow(dead_code)]
    pub fn test_long_word() -> UIComponentRequest {
        UIComponentRequest {
            identifier: "TestLongWord".to_string(),
            position: Point::new(2, 4),
            text: "#[white]This is a really long sentence to test formatting, I love lauren! <3#[]"
                .to_string(),
            z_priority: 1,
            component_type: UIComponentType::Label {
                background: Some(BLACK.into()),
                align: TextAlign::Left,
            },
        }
    }

    pub fn test_rex_image(image_name: &str) -> UIComponentRequest {
        match XpFile::from_resource(format!("../resources/rex/{}.xp", image_name).as_str()).ok() {
            Some(file) => UIComponentRequest {
                identifier: "BaseGameUi".to_string(),
                text: String::new(),
                position: Point::new(50, 5),
                z_priority: 10,
                component_type: UIComponentType::RexDrawing { file },
            },
            None => panic!("Error trying to load resource path for xp files"),
        }
    }

    pub fn town_name_component(town_name: impl ToString) -> Self {
        Self {
            identifier: "Town Name".to_string(),
            text: format!("#[white]{}#[]", town_name.to_string()),
            position: Point::new(25, 3),
            z_priority: 3,
            component_type: UIComponentType::Label {
                background: Some(BLACK.into()),
                align: TextAlign::Center,
            },
        }
    }
}

pub fn initialize_layout(ui: &mut UICreationRequests) {
    ui.add(UIComponentRequest::fps_counter(0))
        .add(UIComponentRequest::test_rex_image("ui"))
        .add(UIComponentRequest::log_message("Hi".to_string(), 1))
        .add(UIComponentRequest::log_message("Hello".to_string(), 2))
        .add(UIComponentRequest::log_message("Yo".to_string(), 3))
        .add(UIComponentRequest::log_message("We're back".to_string(), 4))
        .add(UIComponentRequest::log_message("Sup".to_string(), 5))
        .add(UIComponentRequest::log_message("Why is this sentence on the weird line".to_string(), 6))
        .add(UIComponentRequest::log_message("Sup".to_string(), 7))
        .add(UIComponentRequest::log_message("The most exciting eureka moment I've had was when I realized.".to_string(), 8))
        .add(UIComponentRequest::log_message("Sup".to_string(), 9))
        .add(UIComponentRequest::log_message("Oh wowzers".to_string(), 10))
        .add(UIComponentRequest::town_name_component("The Forest of Testing"));
}

pub fn fps_counter_update(ecs: &World, fps: f32) {
    let mut ui_requests = ecs.fetch_mut::<UILayout>();

    if let Some(fps_counter) = ui_requests.get_mut("FPS") {
        fps_counter.text = format!("#[pink]FPS: {}", fps);
    }
}
