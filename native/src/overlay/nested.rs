use std::cell::RefCell;

use iced_core::{Point, Rectangle, Size};

use crate::{
    event, layout, mouse, overlay, renderer, widget, Clipboard, Event, Layout,
    Overlay, Shell,
};

/// An [`Overlay`] container that displays nested overlays
#[allow(missing_debug_implementations)]
pub struct Nested<'a, Message, Renderer> {
    overlay: Inner<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Nested<'a, Message, Renderer> {
    /// Creates a nested overlay from the provided [`overlay::Element`]
    pub fn new(element: overlay::Element<'a, Message, Renderer>) -> Self {
        Self {
            overlay: Inner(RefCell::new(element)),
        }
    }
}

struct Inner<'a, Message, Renderer>(
    RefCell<overlay::Element<'a, Message, Renderer>>,
);

impl<'a, Message, Renderer> Inner<'a, Message, Renderer> {
    fn with_element_mut<T>(
        &self,
        mut f: impl FnMut(&mut overlay::Element<'_, Message, Renderer>) -> T,
    ) -> T {
        (f)(&mut self.0.borrow_mut())
    }
}

impl<'a, Message, Renderer> Overlay<Message, Renderer>
    for Nested<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            renderer: &Renderer,
            bounds: Size,
            position: Point,
        ) -> Vec<layout::Node>
        where
            Renderer: crate::Renderer,
        {
            let translation = position - Point::ORIGIN;

            let node = element.layout(renderer, bounds, translation);

            if let Some(mut overlay) =
                element.overlay(Layout::new(&node), renderer)
            {
                vec![node]
                    .into_iter()
                    .chain(recurse(&mut overlay, renderer, bounds, position))
                    .collect()
            } else {
                vec![node]
            }
        }

        self.overlay.with_element_mut(|element| {
            layout::Node::with_children(
                bounds,
                recurse(element, renderer, bounds, position),
            )
        })
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &<Renderer as crate::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            renderer: &mut Renderer,
            theme: &<Renderer as crate::Renderer>::Theme,
            style: &renderer::Style,
            cursor_position: Point,
        ) where
            Renderer: crate::Renderer,
        {
            if let Some(layout) = layouts.next() {
                renderer.with_layer(layout.bounds(), |renderer| {
                    element.draw(
                        renderer,
                        theme,
                        style,
                        layout,
                        cursor_position,
                    );
                });

                if let Some(mut overlay) = element.overlay(layout, renderer) {
                    recurse(
                        &mut overlay,
                        layouts,
                        renderer,
                        theme,
                        style,
                        cursor_position,
                    );
                }
            }
        }

        self.overlay.with_element_mut(|element| {
            let layouts = layout.children();

            recurse(element, layouts, renderer, theme, style, cursor_position);
        })
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) where
            Renderer: crate::Renderer,
        {
            if let Some(layout) = layouts.next() {
                element.operate(layout, renderer, operation);

                if let Some(mut overlay) = element.overlay(layout, renderer) {
                    recurse(&mut overlay, layouts, renderer, operation);
                }
            }
        }

        let layouts = layout.children();

        recurse(self.overlay.0.get_mut(), layouts, renderer, operation)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            event: Event,
            cursor_position: Point,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status
        where
            Renderer: crate::Renderer,
        {
            if let Some(layout) = layouts.next() {
                let status = if let Some(mut overlay) =
                    element.overlay(layout, renderer)
                {
                    recurse(
                        &mut overlay,
                        layouts,
                        event.clone(),
                        cursor_position,
                        renderer,
                        clipboard,
                        shell,
                    )
                } else {
                    event::Status::Ignored
                };

                if matches!(status, event::Status::Ignored) {
                    element.on_event(
                        event,
                        layout,
                        cursor_position,
                        renderer,
                        clipboard,
                        shell,
                    )
                } else {
                    status
                }
            } else {
                event::Status::Ignored
            }
        }

        let layouts = layout.children();

        recurse(
            self.overlay.0.get_mut(),
            layouts,
            event,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            cursor_position: Point,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction
        where
            Renderer: crate::Renderer,
        {
            if let Some(layout) = layouts.next() {
                let interaction = if let Some(mut overlay) =
                    element.overlay(layout, renderer)
                {
                    recurse(
                        &mut overlay,
                        layouts,
                        cursor_position,
                        viewport,
                        renderer,
                    )
                } else {
                    mouse::Interaction::default()
                };

                if matches!(interaction, mouse::Interaction::Idle) {
                    element.mouse_interaction(
                        layout,
                        cursor_position,
                        viewport,
                        renderer,
                    )
                } else {
                    interaction
                }
            } else {
                mouse::Interaction::default()
            }
        }

        self.overlay.with_element_mut(|element| {
            let layouts = layout.children();

            recurse(element, layouts, cursor_position, viewport, renderer)
        })
    }

    fn is_over(
        &self,
        layout: Layout<'_>,
        renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            renderer: &Renderer,
            cursor_position: Point,
        ) -> bool
        where
            Renderer: crate::Renderer,
        {
            if let Some(layout) = layouts.next() {
                let is_over =
                    element.is_over(layout, renderer, cursor_position);

                if is_over {
                    return true;
                }

                if let Some(mut overlay) = element.overlay(layout, renderer) {
                    recurse(&mut overlay, layouts, renderer, cursor_position)
                } else {
                    false
                }
            } else {
                false
            }
        }

        self.overlay.with_element_mut(|element| {
            let layouts = layout.children();

            recurse(element, layouts, renderer, cursor_position)
        })
    }

    fn overlay<'b>(
        &'b mut self,
        _layout: crate::Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        None
    }
}
