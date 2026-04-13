use super::prelude::*;

pub fn dashboard_ui(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    engine: &TaskEngine) {
    for (task, status) in engine.task_view() {
        FlexCard::horizontal()
            .show(flex, |flex| task_card(flex, view, task, status));
    }
}

fn task_card(
    flex: &mut FlexInstance,
    view: &mut ViewContext,
    task: &Task,
    status: TaskStatus) {
    // Start — disabled while already running or invalid.
    flex.add(
            item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_PLAY)
                .enabled(
                    status != TaskStatus::Running &&
                    status != TaskStatus::Invalid))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_action(Action::StartTask(task.id)));

    // Stop — disabled when not running.
    flex.add(
            item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_STOP)
                .enabled(status == TaskStatus::Running))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_action(Action::StopTask(task.id)));

    // Task name + status. Clicking anywhere on this opens the task viewer.
    flex
        .add(
            item().grow(1.0),
            Button::new("")
                .left_text(task.name.as_str())
                .right_text(custom::task_status_label(status).small())
                .truncate())
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_navigation(Page::TaskViewer(task.id)));

    // Edit — disabled when running.
    flex
        .add(item(),
            FlexActionButton::new()
                .icon(nf::fa::FA_PEN)
                .enabled(status != TaskStatus::Running))
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
        .then(|| view.set_navigation(Page::TaskEditor(task.clone())));

    // let delete_confirm_id = Id::new(("dashboard.delete_confirm", task.id));
    // let delete_confirm =
    //     flex.ui()
    //         .data_mut(|data| data.get_temp::<bool>(delete_confirm_id))
    //         .unwrap_or(false);
}
