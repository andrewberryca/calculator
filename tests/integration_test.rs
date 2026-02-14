use calculator::CalcApp;

#[test]
fn test_basic_addition() {
    let mut app = CalcApp::new();
    app.input_digit('5');
    app.input_operator('+');
    app.input_digit('3');
    app.compute();
    assert_eq!(app.display, "8");
}

#[test]
fn test_advanced_calculation() {
    let mut app = CalcApp::new();
    // Test: 100 - 25 = 75
    app.input_digit('1');
    app.input_digit('0');
    app.input_digit('0');
    app.input_operator('-');
    app.input_digit('2');
    app.input_digit('5');
    app.compute();
    assert_eq!(app.display, "75");
}

#[test]
fn test_multiple_operations() {
    let mut app = CalcApp::new();
    // Test: 10 + 5 - 3 = 12
    app.input_digit('1');
    app.input_digit('0');
    app.input_operator('+');
    app.input_digit('5');
    app.compute();
    assert_eq!(app.display, "15");
    
    app.input_operator('-');
    app.input_digit('3');
    app.compute();
    assert_eq!(app.display, "12");
}
