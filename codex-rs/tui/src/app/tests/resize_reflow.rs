use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn initial_resume_replay_buffer_keeps_visible_tail_when_resize_row_cap_is_larger() {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    enable_terminal_resize_reflow(&mut app);
    app.config.terminal_resize_reflow.max_rows = TerminalResizeReflowMaxRows::Limit(100);
    app.transcript_cells = (0..8)
        .map(|i| plain_line_cell(format!("cell {i}")))
        .collect();

    let width = 80;
    let terminal_size = ratatui::layout::Size::new(
        width,
        app.chat_widget.desired_height(width) + /*visible_tail_rows*/ 3,
    );

    app.begin_initial_history_replay_buffer();
    for index in 0..8 {
        app.buffer_initial_resume_replay_display_lines(
            vec![Line::from(format!("line {index}")).into()],
            terminal_size,
        );
    }

    let buffer = app
        .initial_history_replay_buffer
        .as_ref()
        .expect("initial replay buffer should remain active");
    assert_eq!(app.transcript_cells.len(), 8);
    assert_eq!(
        buffer
            .retained_lines
            .iter()
            .map(rendered_line_text)
            .collect::<Vec<_>>(),
        vec![
            "line 5".to_string(),
            "line 6".to_string(),
            "line 7".to_string(),
        ]
    );
}

#[tokio::test]
async fn initial_resume_replay_buffer_keeps_at_least_one_visible_tail_row() {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    enable_terminal_resize_reflow(&mut app);
    app.config.terminal_resize_reflow.max_rows = TerminalResizeReflowMaxRows::Limit(100);

    let width = 80;
    let terminal_size = ratatui::layout::Size::new(width, app.chat_widget.desired_height(width));

    app.begin_initial_history_replay_buffer();
    for index in 0..3 {
        app.buffer_initial_resume_replay_display_lines(
            vec![Line::from(format!("line {index}")).into()],
            terminal_size,
        );
    }

    let buffer = app
        .initial_history_replay_buffer
        .as_ref()
        .expect("initial replay buffer should remain active");
    assert_eq!(
        buffer
            .retained_lines
            .iter()
            .map(rendered_line_text)
            .collect::<Vec<_>>(),
        vec!["line 2".to_string()]
    );
}

#[tokio::test]
async fn proposed_plan_consolidation_without_resize_does_not_request_reflow_repair() {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    enable_terminal_resize_reflow(&mut app);
    app.transcript_cells = vec![
        Arc::new(history_cell::new_proposed_plan_stream(
            vec![Line::from("plan head")],
            /*is_stream_continuation*/ false,
        )),
        Arc::new(history_cell::new_proposed_plan_stream(
            vec![Line::from("plan tail")],
            /*is_stream_continuation*/ true,
        )),
    ];

    let consolidation =
        app.consolidate_trailing_proposed_plan_stream_cells("final plan".to_string());

    assert_eq!(consolidation.replaced_range(), Some(0..2));
    assert_eq!(app.transcript_cells.len(), 1);
    assert!(
        app.transcript_cells[0]
            .as_any()
            .is::<history_cell::ProposedPlanCell>()
    );
    assert!(!app.transcript_reflow.take_stream_finish_reflow_needed());
    assert!(!app.transcript_reflow.has_pending_reflow());
}

#[tokio::test]
async fn proposed_plan_consolidation_after_resize_keeps_reflow_repair_request() {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    enable_terminal_resize_reflow(&mut app);
    app.transcript_cells = vec![Arc::new(history_cell::new_proposed_plan_stream(
        vec![Line::from("plan")],
        /*is_stream_continuation*/ false,
    ))];
    app.transcript_reflow.mark_resize_requested_during_stream();

    let consolidation =
        app.consolidate_trailing_proposed_plan_stream_cells("final plan".to_string());

    assert_eq!(consolidation.replaced_range(), Some(0..1));
    assert_eq!(app.transcript_cells.len(), 1);
    assert!(app.transcript_reflow.take_stream_finish_reflow_needed());
}
