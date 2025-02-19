use crate::context::EmitContext;
use crate::{cache::CachedData, prelude::*};
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

pub struct EventHandle<'a, 'b, V> {
    cx: &'a mut EventContext<'b>,
    entity: Entity,
    p: PhantomData<V>,
}

impl<'a, 'b, V> EventHandle<'a, 'b, V> {
    pub fn new(cx: &'a mut EventContext<'b>) -> Self {
        let entity = cx.current();
        Self { cx, entity, p: PhantomData::default() }
    }

    pub fn current(&self) -> Entity {
        self.entity
    }

    pub fn cache(&self) -> &CachedData {
        &self.cx.cache
    }

    /// Sets application focus to the current entity using the previous focus visibility.
    pub fn focus(&mut self) {
        self.cx.focus()
    }

    pub fn play_animation(&mut self, animation: Animation) {
        self.cx.play_animation(animation);
    }

    pub fn modify<F>(&mut self, f: F)
    where
        F: FnOnce(&mut V),
        V: 'static,
    {
        if let Some(view) = self
            .cx
            .views
            .get_mut(&self.entity)
            .and_then(|view_handler| view_handler.downcast_mut::<V>())
        {
            (f)(view);
        }
    }
}

impl<'a, 'b, V> AsMut<EventContext<'b>> for EventHandle<'a, 'b, V> {
    fn as_mut(&mut self) -> &mut EventContext<'b> {
        self.cx
    }
}

impl<'a, 'b, V> DataContext for EventHandle<'a, 'b, V> {
    fn data<T: 'static>(&self) -> Option<&T> {
        // Return data for the static model.
        if let Some(t) = <dyn Any>::downcast_ref::<T>(&()) {
            return Some(t);
        }

        for entity in self.entity.parent_iter(&self.cx.tree) {
            if let Some(model_data_store) = self.cx.data.get(entity) {
                if let Some(model) = model_data_store.models.get(&TypeId::of::<T>()) {
                    return model.downcast_ref::<T>();
                }
            }

            if let Some(view_handler) = self.cx.views.get(&entity) {
                if let Some(data) = view_handler.downcast_ref::<T>() {
                    return Some(data);
                }
            }
        }

        None
    }
}

impl<'a, 'b, V> EmitContext for EventHandle<'a, 'b, V> {
    fn emit<M: Any + Send>(&mut self, message: M) {
        self.cx.emit(message);
    }

    fn emit_to<M: Any + Send>(&mut self, target: Entity, message: M) {
        self.cx.emit_to(target, message);
    }

    fn emit_custom(&mut self, event: Event) {
        self.cx.emit_custom(event);
    }
}

pub(crate) struct ActionsModel<V> {
    pub(crate) on_press: Option<Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>>,
    pub(crate) on_press_down: Option<Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>>,
    pub(crate) on_hover: Option<Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>>,
    pub(crate) on_hover_out: Option<Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>>,
    pub(crate) on_over: Option<Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>>,
    pub(crate) on_over_out: Option<Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>>,
    pub(crate) on_mouse_move: Option<Box<dyn Fn(&mut EventHandle<V>, f32, f32) + Send + Sync>>,
    pub(crate) on_mouse_down: Option<Box<dyn Fn(&mut EventHandle<V>, MouseButton) + Send + Sync>>,
    pub(crate) on_mouse_up: Option<Box<dyn Fn(&mut EventHandle<V>, MouseButton) + Send + Sync>>,
    pub(crate) on_focus_in: Option<Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>>,
    pub(crate) on_focus_out: Option<Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>>,
    pub(crate) on_geo_changed:
        Option<Box<dyn Fn(&mut EventHandle<V>, GeometryChanged) + Send + Sync>>,
}

impl<V> ActionsModel<V> {
    pub(crate) fn new() -> Self {
        Self {
            on_press: None,
            on_press_down: None,
            on_hover: None,
            on_hover_out: None,
            on_over: None,
            on_over_out: None,
            on_mouse_move: None,
            on_mouse_down: None,
            on_mouse_up: None,
            on_focus_in: None,
            on_focus_out: None,
            on_geo_changed: None,
        }
    }
}

impl<V: 'static> Model for ActionsModel<V> {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.take().map(|actions_event| match actions_event {
            ActionsEvent::OnPress(on_press) => {
                self.on_press = Some(on_press);
            }

            ActionsEvent::OnPressDown(on_press_down) => {
                self.on_press_down = Some(on_press_down);
            }

            ActionsEvent::OnHover(on_hover) => {
                self.on_hover = Some(on_hover);
            }

            ActionsEvent::OnHoverOut(on_hover_out) => {
                self.on_hover_out = Some(on_hover_out);
            }

            ActionsEvent::OnOver(on_over) => {
                self.on_over = Some(on_over);
            }

            ActionsEvent::OnOverOut(on_over_out) => {
                self.on_over_out = Some(on_over_out);
            }

            ActionsEvent::OnMouseMove(on_move) => {
                self.on_mouse_move = Some(on_move);
            }

            ActionsEvent::OnMouseDown(on_mouse_down) => {
                self.on_mouse_down = Some(on_mouse_down);
            }

            ActionsEvent::OnMouseUp(on_mouse_up) => {
                self.on_mouse_up = Some(on_mouse_up);
            }

            ActionsEvent::OnFocusIn(on_focus_in) => {
                self.on_focus_in = Some(on_focus_in);
            }

            ActionsEvent::OnFocusOut(on_focus_out) => {
                self.on_focus_out = Some(on_focus_out);
            }

            ActionsEvent::OnGeoChanged(on_geo_changed) => {
                self.on_geo_changed = Some(on_geo_changed);
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::Press { mouse } => {
                let over = if *mouse { cx.hovered() } else { cx.focused() };
                if cx.current() != over && !over.is_descendant_of(cx.tree, cx.current()) {
                    return;
                }

                if let Some(action) = &self.on_press {
                    (action)(&mut EventHandle::<V>::new(cx));
                }
            }

            WindowEvent::PressDown { mouse } => {
                let over = if *mouse { cx.hovered() } else { cx.focused() };
                if cx.current() != over && !over.is_descendant_of(cx.tree, cx.current()) {
                    return;
                }
                if let Some(action) = &self.on_press_down {
                    (action)(&mut EventHandle::<V>::new(cx));
                }
            }

            WindowEvent::MouseEnter => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_hover {
                        (action)(&mut EventHandle::<V>::new(cx));
                    }
                }
            }

            WindowEvent::MouseLeave => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_hover_out {
                        (action)(&mut EventHandle::<V>::new(cx));
                    }
                }
            }

            WindowEvent::MouseOver => {
                if let Some(action) = &self.on_over {
                    (action)(&mut EventHandle::<V>::new(cx));
                }
            }

            WindowEvent::MouseOut => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_over_out {
                        (action)(&mut EventHandle::<V>::new(cx));
                    }
                }
            }

            WindowEvent::MouseMove(x, y) => {
                if let Some(action) = &self.on_mouse_move {
                    (action)(&mut EventHandle::<V>::new(cx), *x, *y);
                }
            }

            WindowEvent::MouseDown(mouse_button) => {
                if let Some(action) = &self.on_mouse_down {
                    (action)(&mut EventHandle::<V>::new(cx), *mouse_button);
                }
            }

            WindowEvent::MouseUp(mouse_button) => {
                if let Some(action) = &self.on_mouse_up {
                    (action)(&mut EventHandle::<V>::new(cx), *mouse_button);
                }
            }

            WindowEvent::FocusIn => {
                if let Some(action) = &self.on_focus_in {
                    (action)(&mut EventHandle::<V>::new(cx));
                }
            }

            WindowEvent::FocusOut => {
                if let Some(action) = &self.on_focus_out {
                    (action)(&mut EventHandle::<V>::new(cx));
                }
            }

            WindowEvent::GeometryChanged(geo) => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_geo_changed {
                        (action)(&mut EventHandle::<V>::new(cx), *geo);
                    }
                }
            }

            _ => {}
        });
    }
}

pub(crate) enum ActionsEvent<V> {
    OnPress(Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>),
    OnPressDown(Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>),
    OnHover(Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>),
    OnHoverOut(Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>),
    OnOver(Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>),
    OnOverOut(Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>),
    OnMouseMove(Box<dyn Fn(&mut EventHandle<V>, f32, f32) + Send + Sync>),
    OnMouseDown(Box<dyn Fn(&mut EventHandle<V>, MouseButton) + Send + Sync>),
    OnMouseUp(Box<dyn Fn(&mut EventHandle<V>, MouseButton) + Send + Sync>),
    OnFocusIn(Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>),
    OnFocusOut(Box<dyn Fn(&mut EventHandle<V>) + Send + Sync>),
    OnGeoChanged(Box<dyn Fn(&mut EventHandle<V>, GeometryChanged) + Send + Sync>),
}

/// Modifiers which add an action callback to a view.
pub trait ActionModifiers<V> {
    /// Adds a callback which is performed when the the view receives the [`Press`](crate::prelude::WindowEvent::Press) event.
    /// By default a view receives the [`Press`](crate::prelude::WindowEvent::Press) event when the left mouse button is pressed and then released on the view,
    /// or when the space or enter keys are pressed and then released while the view is focused.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_press(|_| println!("View was pressed!"));
    /// ```
    fn on_press<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync;

    /// Adds a callback which is performed when the the view receives the [`PressDown`](crate::prelude::WindowEvent::PressDown) event.
    // By default a view receives the [`PressDown`](crate::prelude::WindowEvent::PressDown) event when the left mouse button is pressed on the view,
    /// or when the space or enter keys are pressed while the view is focused.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_press_down(|_| println!("View was pressed down!"));
    /// ```
    fn on_press_down<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves over a view.
    /// This callback is not triggered when the mouse pointer moves over an overlapping child of the view.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_hover(|_| println!("Mouse cursor entered the view!"));
    /// ```
    fn on_hover<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves away from a view.
    /// This callback is not triggered when the mouse pointer moves away from an overlapping child of the view.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_hover_out(|_| println!("Mouse cursor left the view!"));
    /// ```
    fn on_hover_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves over the bounds of a view,
    /// including any overlapping children.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_over(|_| println!("Mouse cursor entered the view bounds!"));
    /// ```
    fn on_over<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves away from the bounds of a view,
    /// including any overlapping children.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_over_out(|_| println!("Mouse cursor left the view bounds!"));
    /// ```
    fn on_over_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves within the bounds of a view.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_mouse_move(|_, x, y| println!("Cursor moving: {} {}", x, y));
    /// ```
    fn on_mouse_move<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>, f32, f32) + Send + Sync;

    /// Adds a callback which is performed when a mouse button is pressed on the view.
    /// Unlike the `on_press` callback, this callback is triggered for all mouse buttons and not for any keyboard keys.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_mouse_down(|_, button| println!("Mouse button, {:?}, was pressed!", button));
    /// ```
    fn on_mouse_down<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>, MouseButton) + Send + Sync;

    /// Adds a callback which is performed when a mouse button is released on the view.
    /// Unlike the `on_release` callback, this callback is triggered for all mouse buttons and not for any keyboard keys.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_mouse_up(|_, button| println!("Mouse button, {:?}, was released!", button));
    /// ```
    fn on_mouse_up<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>, MouseButton) + Send + Sync;

    /// Adds a callback which is performed when the view gains keyboard focus.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_focus_in(|_| println!("View gained keyboard focus!"));
    /// ```
    fn on_focus_in<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync;

    /// Adds a callback which is performed when the view loses keyboard focus.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_focus_out(|_| println!("View lost keyboard focus!"));
    /// ```
    fn on_focus_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync;

    /// Adds a callback which is performed when the the view changes size or position after layout.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::default();
    /// Element::new(cx).on_geo_changed(|_, _| println!("View geometry changed!"));
    /// ```
    fn on_geo_changed<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>, GeometryChanged) + Send + Sync;
}

// If the entity doesn't have an `ActionsModel` then add one to the entity
fn build_action_model<V: 'static>(cx: &mut Context, entity: Entity) {
    if cx
        .data
        .get(entity)
        .and_then(|model_data_store| model_data_store.models.get(&TypeId::of::<ActionsModel<V>>()))
        .is_none()
    {
        cx.with_current(entity, |cx| {
            ActionsModel::<V>::new().build(cx);
        });
    }
}

impl<'a, V: View> ActionModifiers<V> for Handle<'a, V> {
    fn on_press<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnPress(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_press_down<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnPressDown(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_hover<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnHover(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_hover_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnHoverOut(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_over<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnOver(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_over_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnOverOut(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_mouse_move<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>, f32, f32) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnMouseMove(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_mouse_down<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>, MouseButton) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnMouseDown(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_mouse_up<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>, MouseButton) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnMouseUp(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_focus_in<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnFocusIn(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_focus_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnFocusOut(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_geo_changed<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventHandle<V>, GeometryChanged) + Send + Sync,
    {
        build_action_model::<V>(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnGeoChanged(Box::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }
}
