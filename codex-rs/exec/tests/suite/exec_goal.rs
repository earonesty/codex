#![cfg(not(target_os = "windows"))]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use core_test_support::responses;
use core_test_support::test_codex_exec::test_codex_exec;
use predicates::str::contains;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_goal_sets_active_goal_before_initial_turn() -> anyhow::Result<()> {
    let test = test_codex_exec();
    let server = responses::start_mock_server().await;
    let body = responses::sse(vec![
        responses::ev_response_created("resp1"),
        responses::ev_assistant_message("m1", "fixture hello"),
        responses::ev_completed("resp1"),
    ]);
    let response_mock = responses::mount_sse_once(&server, body).await;

    test.cmd_with_server(&server)
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(test.cwd_path())
        .arg("-m")
        .arg("gpt-5.1")
        .arg("--goal")
        .arg("ship the exec goal flag")
        .arg("start the implementation")
        .assert()
        .success();

    let request = response_mock.single_request();
    assert!(
        request.has_message_with_input_texts("user", |texts| {
            texts == ["start the implementation".to_string()]
        }),
        "request should preserve the initial user prompt"
    );
    assert!(
        request.body_contains_text("ship the exec goal flag"),
        "request should include the active goal context before the first model turn"
    );

    Ok(())
}

#[test]
fn exec_goal_rejects_ephemeral_threads() {
    let test = test_codex_exec();

    test.cmd()
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(test.cwd_path())
        .arg("--ephemeral")
        .arg("--goal")
        .arg("ship the exec goal flag")
        .arg("start the implementation")
        .assert()
        .code(1)
        .stderr(contains("--goal cannot be used with --ephemeral"));
}

#[test]
fn exec_goal_rejects_subcommands() {
    let test = test_codex_exec();

    test.cmd()
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(test.cwd_path())
        .arg("--goal")
        .arg("ship the exec goal flag")
        .arg("review")
        .assert()
        .code(1)
        .stderr(contains(
            "--goal can only be used when starting a new exec thread",
        ));
}
