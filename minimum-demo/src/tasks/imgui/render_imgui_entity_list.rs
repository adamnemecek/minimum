use minimum::resource::{DataRequirement, Read, Write};
use minimum::{
    ComponentStorage, EntityFactory, EntitySet, PendingDeleteComponent, Task, TaskContext,
    WriteComponent,
};

use crate::resources::{EditorUiState, ImguiManager, InputManager, TimeState};

use crate::components::EditorSelectedComponent;
use named_type::NamedType;

use crate::framework::CloneComponentPrototype;
use crate::framework::FrameworkEntityPrototype;

#[derive(NamedType)]
pub struct RenderImguiEntityList;
impl Task for RenderImguiEntityList {
    type RequiredResources = (
        Read<TimeState>,
        Write<ImguiManager>,
        Read<EditorUiState>,
        Read<EntitySet>,
        Write<EntityFactory>,
        WriteComponent<EditorSelectedComponent>,
        Read<InputManager>,
        WriteComponent<PendingDeleteComponent>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::AUTHORITY_CLIENT as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            time_state,
            mut imgui_manager,
            editor_ui_state,
            entity_set,
            mut entity_factory,
            mut editor_selected_components,
            input_manager,
            mut pending_delete_components,
        ) = data;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            use imgui::im_str;

            use imgui::ImGuiSelectableFlags;

            let window_options = editor_ui_state.window_options(time_state.play_mode);

            if window_options.show_entity_list {
                ui.window(im_str!("Entity List"))
                    .position([0.0, 50.0], imgui::Condition::Once)
                    .size([200.0, 200.0], imgui::Condition::Once)
                    .build(|| {
                        let add_entity = ui.button(im_str!("\u{e8b1} Add"), [80.0, 0.0]);
                        ui.same_line_with_spacing(80.0, 10.0);
                        let remove_entity = ui.button(im_str!("\u{e897} Delete"), [80.0, 0.0]);

                        if add_entity {
                            editor_selected_components.free_all();
                            let pec = FrameworkEntityPrototype::new(
                                std::path::PathBuf::from("testpath"),
                                vec![Box::new(CloneComponentPrototype::new(
                                    EditorSelectedComponent::new(),
                                ))],
                            );
                            entity_factory.enqueue_create(Box::new(pec));
                        }

                        if remove_entity {
                            for (entity_handle, _c) in editor_selected_components.iter(&entity_set)
                            {
                                entity_set
                                    .enqueue_free(&entity_handle, &mut *pending_delete_components);
                            }
                        }

                        let name = im_str!("");
                        if unsafe {
                            imgui_sys::igListBoxHeaderVec2(
                                name.as_ptr(),
                                imgui_sys::ImVec2 { x: -1.0, y: -1.0 },
                            )
                        } {
                            for entity in entity_set.iter() {
                                let is_selected = if let Some(_selected_component) =
                                    editor_selected_components.get(&entity.handle())
                                {
                                    true
                                } else {
                                    false
                                };

                                let s = im_str!("{:?}", entity.handle());
                                let clicked = ui.selectable(
                                    &s,
                                    is_selected,
                                    ImGuiSelectableFlags::empty(),
                                    [0.0, 0.0],
                                );

                                if clicked {
                                    let is_control_held = input_manager
                                        .is_key_down(winit::event::VirtualKeyCode::LControl);
                                    if is_control_held {
                                        if !editor_selected_components.exists(&entity.handle()) {
                                            editor_selected_components.allocate(
                                                &entity.handle(),
                                                EditorSelectedComponent::new(),
                                            );
                                        } else {
                                            editor_selected_components.free(&entity.handle());
                                        }
                                    } else {
                                        editor_selected_components.free_all();
                                        if !editor_selected_components.exists(&entity.handle()) {
                                            editor_selected_components.allocate(
                                                &entity.handle(),
                                                EditorSelectedComponent::new(),
                                            );
                                        }
                                    }
                                }
                            }

                            unsafe {
                                imgui_sys::igListBoxFooter();
                            }
                        }
                    });
            }
        })
    }
}