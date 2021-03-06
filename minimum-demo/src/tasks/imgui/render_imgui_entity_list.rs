use crate::base::resource::{DataRequirement, Read, Write};
use crate::base::{
    ComponentStorage, EntitySet, ResourceTaskImpl, TaskConfig, TaskContextFlags, WriteComponent,
};

use crate::framework::resources::InputState;
use crate::resources::ImguiManager;
#[cfg(feature = "editor")]
use crate::framework::resources::editor::{EditorActionQueue, EditorUiState};
use crate::framework::resources::TimeState;
use crate::framework::resources::FrameworkOptions;

#[cfg(feature = "editor")]
use crate::framework::components::editor::EditorSelectedComponent;

pub struct RenderImguiEntityList;
pub type RenderImguiEntityListTask = crate::base::ResourceTask<RenderImguiEntityList>;
impl ResourceTaskImpl for RenderImguiEntityList {
    type RequiredResources = (
        Read<TimeState>,
        Write<ImguiManager>,
        Read<EditorUiState>,
        Read<EntitySet>,
        WriteComponent<EditorSelectedComponent>,
        Read<InputState>,
        Write<EditorActionQueue>,
        Read<FrameworkOptions>
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePreRender>();
        config.run_only_if(crate::framework::context_flags::AUTHORITY_CLIENT);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            time_state,
            mut imgui_manager,
            editor_ui_state,
            entity_set,
            mut editor_selected_components,
            input_state,
            mut editor_action_queue,
            framework_options
        ) = data;

        imgui_manager.with_ui(|ui: &mut imgui::Ui| {
            use imgui::im_str;

            let window_options = editor_ui_state.window_options(time_state.play_mode);

            if window_options.show_entity_list {
                imgui::Window::new(im_str!("Entity List"))
                    .position([0.0, 50.0], imgui::Condition::Once)
                    .size([350.0, 300.0], imgui::Condition::Once)
                    .build(ui, || {
                        let add_entity = ui.button(im_str!("\u{e8b1} Add"), [80.0, 0.0]);
                        ui.same_line_with_spacing(80.0, 10.0);
                        let remove_entity = ui.button(im_str!("\u{e897} Delete"), [80.0, 0.0]);

                        if add_entity {
                            editor_action_queue.enqueue_add_new_entity();
                        }

                        if remove_entity {
                            editor_action_queue.enqueue_delete_selected_entities();
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
                                let clicked =
                                    imgui::Selectable::new(&s).selected(is_selected).build(ui);

                                if clicked {
                                    let is_control_held =
                                        input_state.is_key_down(framework_options.keybinds.modify_imgui_entity_list_modify_selection_add1) ||
                                        input_state.is_key_down(framework_options.keybinds.modify_imgui_entity_list_modify_selection_add2);
                                    if is_control_held {
                                        if !editor_selected_components.exists(&entity.handle()) {
                                            editor_selected_components.allocate(
                                                &entity.handle(),
                                                EditorSelectedComponent::new(),
                                            ).unwrap();
                                        } else {
                                            editor_selected_components.free(&entity.handle());
                                        }
                                    } else {
                                        editor_selected_components.free_all();
                                        if !editor_selected_components.exists(&entity.handle()) {
                                            editor_selected_components.allocate(
                                                &entity.handle(),
                                                EditorSelectedComponent::new(),
                                            ).unwrap();
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
