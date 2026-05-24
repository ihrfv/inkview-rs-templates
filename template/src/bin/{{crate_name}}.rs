{% if framework == "slint" %}
use std::{cell::RefCell, rc::Rc};

use inkview::Event;
use slint::{ComponentHandle, Model};

mod ui {
    slint::include_modules!();
}

fn main() {
    let iv = Box::leak(Box::new(inkview::load())) as &_;

    let (evt_tx, evt_rx) = std::sync::mpsc::channel();

    std::thread::spawn({
        move || {
            if evt_rx.recv().unwrap() != Event::Init {
                panic!("expected init event first");
            }

            // I hope this thing lives as long as the process
            let screen = inkview::screen::Screen::new(iv);

            let display = inkview_slint::Backend::new(screen, evt_rx);

            slint::platform::set_platform(Box::new(display)).unwrap();

            let window = Rc::new(ui::MainWindow::new().unwrap());

            let model = Rc::new(slint::VecModel::default());
            window.set_model(model.clone().into());

            let undo_stack;
            {
                let model = model.clone();
                undo_stack = Rc::new(RefCell::new(UndoStack::new(move |change| match change {
                    Change::CircleAdded { row } => {
                        let circle = model.row_data(row).unwrap();
                        model.remove(row);
                        Change::CircleRemoved { row, circle }
                    }
                    Change::CircleRemoved { row, circle } => {
                        model.insert(row, circle);
                        Change::CircleAdded { row }
                    }
                    Change::CircleResized { row, old_d } => {
                        let mut circle = model.row_data(row).unwrap();
                        let d = circle.d;
                        circle.d = old_d;
                        model.set_row_data(row, circle);
                        Change::CircleResized { row, old_d: d }
                    }
                })));
            }

            {
                let model = model.clone();
                let undo_stack = undo_stack.clone();
                let window_weak = window.as_weak();
                window.on_background_clicked(move |x, y| {
                    println!("clicked at {x}, {y}");
                    let mut undo_stack = undo_stack.borrow_mut();
                    let window = window_weak.unwrap();

                    model.push(ui::Circle { x, y, d: 30.0 });
                    undo_stack.push(Change::CircleAdded {
                        row: model.row_count() - 1,
                    });

                    window.set_undoable(undo_stack.undoable());
                    window.set_redoable(undo_stack.redoable());
                });
            }

            {
                let undo_stack = undo_stack.clone();
                let window_weak = window.as_weak();
                window.on_undo_clicked(move || {
                    let mut undo_stack = undo_stack.borrow_mut();
                    let window = window_weak.unwrap();
                    undo_stack.undo();
                    window.set_undoable(undo_stack.undoable());
                    window.set_redoable(undo_stack.redoable());
                });
            }

            {
                let undo_stack = undo_stack.clone();
                let window_weak = window.as_weak();
                window.on_redo_clicked(move || {
                    let mut undo_stack = undo_stack.borrow_mut();
                    let window = window_weak.unwrap();
                    undo_stack.redo();
                    window.set_undoable(undo_stack.undoable());
                    window.set_redoable(undo_stack.redoable());
                });
            }

            {
                let model = model.clone();
                let undo_stack = undo_stack.clone();
                let window_weak = window.as_weak();
                window.on_circle_resized(move |row, diameter| {
                    let row = row as usize;
                    let mut undo_stack = undo_stack.borrow_mut();
                    let window = window_weak.unwrap();

                    let mut circle = model.row_data(row).unwrap();
                    let old_d = circle.d;
                    circle.d = diameter;
                    model.set_row_data(row, circle);
                    undo_stack.push(Change::CircleResized { row, old_d });

                    window.set_undoable(undo_stack.undoable());
                    window.set_redoable(undo_stack.redoable());
                });
            }

            window.run().unwrap();
        }
    });

    inkview::iv_main(iv, {
        move |evt| {
            // println!("got evt: {:?}", evt);

            if evt_tx.send(evt).is_err() {
                unsafe {
                    iv.CloseApp();
                }
            }

            Some(())
        }
    })
}

#[allow(clippy::enum_variant_names)]
enum Change {
    CircleAdded { row: usize },
    CircleRemoved { row: usize, circle: ui::Circle },
    CircleResized { row: usize, old_d: f32 },
}

struct UndoStack<F> {
    stack: Vec<Option<Change>>,
    // Everything at and after this is a redo action
    redo_offset: usize,
    undo2redo: F,
}

impl<F> UndoStack<F>
where
    F: Fn(Change) -> Change,
{
    fn new(undo2redo: F) -> Self {
        Self {
            stack: Vec::new(),
            redo_offset: 0,
            undo2redo,
        }
    }

    fn push(&mut self, change: Change) {
        self.stack.truncate(self.redo_offset);
        self.stack.push(Some(change));
        self.redo_offset += 1;
    }

    fn undoable(&self) -> bool {
        self.redo_offset > 0
    }

    fn redoable(&self) -> bool {
        self.redo_offset < self.stack.len()
    }

    fn undo(&mut self) {
        self.redo_offset -= 1;

        let undo = self
            .stack
            .get_mut(self.redo_offset)
            .unwrap()
            .take()
            .unwrap();
        let redo = (self.undo2redo)(undo);
        self.stack[self.redo_offset] = Some(redo);
    }

    fn redo(&mut self) {
        let redo = self
            .stack
            .get_mut(self.redo_offset)
            .unwrap()
            .take()
            .unwrap();
        let undo = (self.undo2redo)(redo);
        self.stack[self.redo_offset] = Some(undo);

        self.redo_offset += 1;
    }
}

{% elsif framework == "embedded-graphics" %}
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics_core::pixelcolor::Gray8;
use inkview::Event;
use inkview_eg::InkviewDisplay;
use std::cell::OnceCell;
use std::convert::Infallible;

fn main() {
    let (event_tx, event_rx) = std::sync::mpsc::channel::<inkview::Event>();
    let iv = Box::leak(Box::new(inkview::load())) as &_;

    std::thread::spawn(move || {
        let mut display = OnceCell::new();

        loop {
            let event = match event_rx.recv() {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Receiving inkview event failed, Err: {e:?}");
                    break;
                }
            };
            match event {
                Event::Init => {
                    // Create a new inkview display which implements [embedded_graphics_core::DrawTarget]
                    let _ = display.set(InkviewDisplay::new(&iv));
                    display.get_mut().unwrap().iv_screen_mut().clear();
                }
                Event::Show | Event::Repaint => {
                    draw_content(display.get_mut().unwrap()).unwrap();
                    display.get_mut().unwrap().flush();
                }
                Event::KeyDown { .. } | Event::Exit => break,
                _ => {}
            }
        }

        unsafe { iv.CloseApp() }
    });

    inkview::iv_main(&iv, move |event| {
        if let Err(e) = event_tx.send(event) {
            eprintln!("Sending inkview event failed, Err: {e:?}");
        }
        Some(())
    });
}

fn draw_content(
    display: &mut impl DrawTarget<Color = Gray8, Error = Infallible>,
) -> anyhow::Result<()> {
    // Create styles used by the drawing operations.
    let thin_stroke = PrimitiveStyle::with_stroke(Gray8::new(0x00), 1);
    let thick_stroke = PrimitiveStyle::with_stroke(Gray8::new(0x00), 3);
    let border_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(Gray8::new(0x00))
        .stroke_width(3)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();
    let fill = PrimitiveStyle::with_fill(Gray8::new(0x00));
    let character_style = MonoTextStyle::new(&FONT_6X10, Gray8::new(0x00));

    let yoffset = 10;

    // Draw a 3px wide outline around the display.
    display
        .bounding_box()
        .into_styled(border_stroke)
        .draw(display)?;

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(display)?;

    // Draw a filled square
    Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
        .into_styled(fill)
        .draw(display)?;

    // Draw a circle with a 3px wide stroke.
    Circle::new(Point::new(88, yoffset), 17)
        .into_styled(thick_stroke)
        .draw(display)?;

    // Draw centered text.
    let text = "embedded-graphics";
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(display)?;

    Ok(())
}

{% else %}
use inkview::bindings::APPLICATION_ATTRIBUTE_APPLICATION_READER;
use inkview::{Event, bindings};
use std::ffi::{CString, c_int};

fn main() {
    let iv = Box::leak(Box::new(inkview::load())) as &_;
    const FONT_SIZE: c_int = 42;

    inkview::iv_main(&iv, move |event| {
        match event {
            Event::Init => unsafe {
                iv.SetCurrentApplicationAttribute(APPLICATION_ATTRIBUTE_APPLICATION_READER, 1);

                let font_name = CString::new("LiberationSans").unwrap();
                let text = CString::new("Hello world!").unwrap();

                let font = iv.OpenFont(font_name.as_ptr(), FONT_SIZE, 0);
                iv.ClearScreen();

                iv.SetFont(font, bindings::BLACK as c_int);
                iv.DrawLine(
                    25,
                    iv.ScreenHeight() - 25,
                    iv.ScreenWidth() - 25,
                    iv.ScreenHeight() - 25,
                    0x00666666,
                );
                iv.FillArea(
                    50,
                    250,
                    iv.ScreenWidth() - 50 * 2,
                    iv.ScreenHeight() - 250 * 2,
                    0x00E0E0E0,
                );
                iv.FillArea(
                    100,
                    300,
                    iv.ScreenWidth() - 100 * 2,
                    iv.ScreenHeight() - 300 * 2,
                    0x00A0A0A0,
                );
                iv.DrawTextRect(
                    0,
                    iv.ScreenHeight() / 2 - FONT_SIZE / 2,
                    iv.ScreenWidth(),
                    FONT_SIZE,
                    text.as_ptr(),
                    bindings::ALIGN_CENTER as c_int,
                );

                // Copies the buffer to the real screen
                iv.FullUpdate();

                iv.CloseFont(font);
            },
            Event::KeyDown { .. } => unsafe {
                iv.CloseApp();
            },
            _ => {}
        }
        Some(())
    });
}

{% endif %}
