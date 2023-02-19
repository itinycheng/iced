use iced_core::{Point, Rectangle, Size};

use crate::{
    event, layout, mouse, overlay, renderer, widget, Clipboard, Event, Layout,
    Shell,
};

/// An [`Overlay`] container that displays nested overlays
#[allow(missing_debug_implementations)]
pub struct Nested<'a, Message, Renderer> {
    overlay: overlay::Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Nested<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Creates a nested overlay from the provided [`overlay::Element`]
    pub fn new(element: overlay::Element<'a, Message, Renderer>) -> Self {
        Self { overlay: element }
    }

    pub fn layout(
        &mut self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            renderer: &Renderer,
            bounds: Size,
            position: Point,
        ) -> layout::Node
        where
            Renderer: crate::Renderer,
        {
            let translation = position - Point::ORIGIN;

            let node = element.layout(renderer, bounds, translation);

            if let Some(mut nested) =
                element.overlay(Layout::new(&node), renderer)
            {
                layout::Node::with_children(
                    node.size(),
                    vec![
                        node,
                        recurse(&mut nested, renderer, bounds, position),
                    ],
                )
            } else {
                layout::Node::with_children(node.size(), vec![node])
            }
        }

        recurse(&mut self.overlay, renderer, bounds, position)
    }

    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        theme: &<Renderer as crate::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            renderer: &mut Renderer,
            theme: &<Renderer as crate::Renderer>::Theme,
            style: &renderer::Style,
            cursor_position: Point,
        ) where
            Renderer: crate::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let nested_layout = layouts.next();

                let is_over = nested_layout
                    .and_then(|nested_layout| {
                        element.overlay(layout, renderer).map(|nested| {
                            nested.is_over(
                                nested_layout,
                                renderer,
                                cursor_position,
                            )
                        })
                    })
                    .unwrap_or_default();

                renderer.with_layer(layout.bounds(), |renderer| {
                    let cursor_position = if is_over {
                        Point::new(-1.0, -1.0)
                    } else {
                        cursor_position
                    };

                    element.draw(
                        renderer,
                        theme,
                        style,
                        layout,
                        cursor_position,
                    );
                });

                if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(nested_layout)
                {
                    recurse(
                        &mut nested,
                        nested_layout,
                        renderer,
                        theme,
                        style,
                        cursor_position,
                    );
                }
            }
        }

        recurse(
            &mut self.overlay,
            layout,
            renderer,
            theme,
            style,
            cursor_position,
        );
    }

    pub fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) where
            Renderer: crate::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                element.operate(layout, renderer, operation);

                if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(&mut nested, nested_layout, renderer, operation);
                }
            }
        }

        recurse(&mut self.overlay, layout, renderer, operation)
    }

    pub fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            event: Event,
            cursor_position: Point,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> (event::Status, bool)
        where
            Renderer: crate::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let (nested_status, nested_is_over) =
                    if let Some((mut nested, nested_layout)) =
                        element.overlay(layout, renderer).zip(layouts.next())
                    {
                        recurse(
                            &mut nested,
                            nested_layout,
                            event.clone(),
                            cursor_position,
                            renderer,
                            clipboard,
                            shell,
                        )
                    } else {
                        (event::Status::Ignored, false)
                    };

                if matches!(nested_status, event::Status::Ignored) {
                    let is_over = nested_is_over
                        || element.is_over(layout, renderer, cursor_position);

                    let cursor_position = if nested_is_over {
                        Point::new(-1.0, -1.0)
                    } else {
                        cursor_position
                    };

                    (
                        element.on_event(
                            event,
                            layout,
                            cursor_position,
                            renderer,
                            clipboard,
                            shell,
                        ),
                        is_over,
                    )
                } else {
                    (nested_status, nested_is_over)
                }
            } else {
                (event::Status::Ignored, false)
            }
        }

        let (status, _) = recurse(
            &mut self.overlay,
            layout,
            event,
            cursor_position,
            renderer,
            clipboard,
            shell,
        );

        status
    }

    pub fn mouse_interaction(
        &mut self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            cursor_position: Point,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction
        where
            Renderer: crate::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let interaction = if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(
                        &mut nested,
                        nested_layout,
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

        recurse(
            &mut self.overlay,
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    pub fn is_over(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            renderer: &Renderer,
            cursor_position: Point,
        ) -> bool
        where
            Renderer: crate::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let is_over =
                    element.is_over(layout, renderer, cursor_position);

                if is_over {
                    return true;
                }

                if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(
                        &mut nested,
                        nested_layout,
                        renderer,
                        cursor_position,
                    )
                } else {
                    false
                }
            } else {
                false
            }
        }

        recurse(&mut self.overlay, layout, renderer, cursor_position)
    }
}
