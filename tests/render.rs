use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

fn fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .expect("read fixture")
}

fn run(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_takumi-cli"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn takumi-cli");

    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin)
        .expect("write stdin");

    child.wait_with_output().expect("wait for takumi-cli")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn renders_the_invoice_template_shape_deterministically() {
    let html = fixture("invoice.html");
    let first = run(&[], &html);
    let second = run(&[], &html);

    assert!(first.status.success(), "{}", stderr(&first));
    assert!(first.stdout.starts_with(b"%PDF-"), "not a pdf");
    assert!(
        first.stderr.is_empty(),
        "unexpected stderr: {}",
        stderr(&first)
    );
    assert_eq!(first.stdout, second.stdout, "nondeterministic output");
}

#[test]
fn writes_the_same_bytes_to_output_file() {
    let html = fixture("invoice.html");
    let dir = std::env::temp_dir().join(format!("takumi-cli-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("invoice.pdf");

    let to_stdout = run(&[], &html);
    let to_file = run(&["--output", path.to_str().expect("utf-8 path")], &html);
    let written = fs::read(&path).expect("read output file");
    fs::remove_dir_all(&dir).expect("clean temp dir");

    assert!(to_file.status.success(), "{}", stderr(&to_file));
    assert!(
        to_file.stdout.is_empty(),
        "stdout should stay empty with --output"
    );
    assert_eq!(written, to_stdout.stdout);
}

#[test]
fn page_geometry_flags_change_the_page_box() {
    let html = fixture("invoice.html");
    let a4 = run(&[], &html);
    let letter = run(
        &["--page-size", "letter", "--landscape", "--margin", "0.5in"],
        &html,
    );

    assert!(letter.status.success(), "{}", stderr(&letter));
    assert_ne!(a4.stdout, letter.stdout);
    // 11in x 8.5in in pt, as krilla writes the MediaBox.
    assert!(
        String::from_utf8_lossy(&letter.stdout).contains("792 612"),
        "expected a landscape letter media box"
    );
}

#[test]
fn missing_glyphs_exit_with_code_3() {
    let output = run(&[], &fixture("missing-glyphs.html"));

    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(output.stdout.is_empty(), "no partial pdf on failure");
    assert!(
        stderr(&output).contains("No registered font covers"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn remote_images_render_with_a_warning() {
    let output = run(&[], &fixture("remote-image.html"));

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stdout.starts_with(b"%PDF-"));
    assert_eq!(
        stderr(&output),
        "takumi-cli: warning: remote image not fetched: https://example.com/logo.png\n"
    );
}

#[test]
fn bad_flags_exit_with_code_2() {
    for args in [
        &["--page-size", "a7"][..],
        &["--margin", "wide"][..],
        &["--fonts-dir", "/definitely/not/here"][..],
    ] {
        let output = run(args, b"<p>x</p>");

        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?}: {}",
            stderr(&output)
        );
        assert!(
            stderr(&output).starts_with("takumi-cli: "),
            "{}",
            stderr(&output)
        );
    }
}
