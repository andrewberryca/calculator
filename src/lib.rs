// Calculator library with core logic and tests
use std::fs;
use std::path::PathBuf;

const MAX_HISTORY: usize = 10;

pub fn history_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_default();
    path.set_file_name("calc_history.txt");
    path
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_path();
    let Ok(contents) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    contents
        .lines()
        .filter_map(|line| {
            let (expr, result) = line.split_once('\t')?;
            Some(HistoryEntry {
                expression: expr.to_string(),
                result: result.to_string(),
            })
        })
        .collect()
}

pub fn save_history(history: &[HistoryEntry]) {
    let path = history_path();
    let content: String = history
        .iter()
        .map(|e| format!("{}\t{}", e.expression, e.result))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(&path, content);
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
}

pub struct CalcApp {
    pub display: String,
    pub expression: String,
    pub first_operand: Option<f64>,
    pub operator: Option<char>,
    pub waiting_for_second: bool,
    pub just_computed: bool,
    pub history: Vec<HistoryEntry>,
    pub show_history: bool,
}

impl CalcApp {
    pub fn new() -> Self {
        Self {
            display: "0".to_string(),
            expression: String::new(),
            first_operand: None,
            operator: None,
            waiting_for_second: false,
            just_computed: false,
            history: load_history(),
            show_history: false,
        }
    }

    pub fn add_history(&mut self, expression: String, result: String) {
        self.history.push(HistoryEntry { expression, result });
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
        save_history(&self.history);
    }

    pub fn input_digit(&mut self, d: char) {
        if self.just_computed {
            self.clear_state();
            self.just_computed = false;
        }
        if self.waiting_for_second {
            self.display = d.to_string();
            self.waiting_for_second = false;
        } else if self.display == "0" {
            self.display = d.to_string();
        } else {
            self.display.push(d);
        }
    }

    pub fn input_dot(&mut self) {
        if self.just_computed {
            self.clear_state();
            self.just_computed = false;
        }
        if self.waiting_for_second {
            self.display = "0.".to_string();
            self.waiting_for_second = false;
        } else if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    pub fn op_symbol(op: char) -> &'static str {
        match op {
            '+' => "+",
            '-' => "-",
            '*' => "\u{00D7}",
            '/' => "\u{00F7}",
            _ => "?",
        }
    }

    pub fn input_operator(&mut self, op: char) {
        if self.display == "Error" {
            return;
        }
        if let Ok(val) = self.display.parse::<f64>() {
            if self.first_operand.is_some() && !self.waiting_for_second {
                self.compute();
                if self.display == "Error" {
                    return;
                }
            }
            let current: f64 = self.display.parse().unwrap_or(val);
            self.expression = format!("{} {}", format_number(current), Self::op_symbol(op));
            self.first_operand = Some(current);
            self.operator = Some(op);
            self.waiting_for_second = true;
            self.just_computed = false;
        }
    }

    pub fn compute(&mut self) {
        if let (Some(a), Some(op)) = (self.first_operand, self.operator) {
            if let Ok(b) = self.display.parse::<f64>() {
                let expr = format!(
                    "{} {} {}",
                    format_number(a),
                    Self::op_symbol(op),
                    format_number(b)
                );
                self.expression = format!("{} =", expr);
                let result = match op {
                    '+' => Some(a + b),
                    '-' => Some(a - b),
                    '*' => Some(a * b),
                    '/' => {
                        if b == 0.0 {
                            None
                        } else {
                            Some(a / b)
                        }
                    }
                    _ => Some(0.0),
                };
                match result {
                    Some(r) => {
                        let result_str = format_number(r);
                        self.add_history(expr, result_str.clone());
                        self.display = result_str;
                    }
                    None => {
                        self.display = "Error".to_string();
                        self.expression = "Cannot divide by zero".to_string();
                    }
                }
                self.first_operand = None;
                self.operator = None;
                self.waiting_for_second = false;
                self.just_computed = true;
            }
        }
    }

    pub fn clear_state(&mut self) {
        self.display = "0".to_string();
        self.expression.clear();
        self.first_operand = None;
        self.operator = None;
        self.waiting_for_second = false;
        self.just_computed = false;
    }

    pub fn clear(&mut self) {
        self.clear_state();
    }

    pub fn clear_entry(&mut self) {
        self.display = "0".to_string();
    }

    pub fn backspace(&mut self) {
        if self.display == "Error" || self.just_computed {
            return;
        }
        if self.display.len() > 1 {
            self.display.pop();
        } else {
            self.display = "0".to_string();
        }
    }

    pub fn toggle_sign(&mut self) {
        if self.display == "Error" || self.display == "0" {
            return;
        }
        if self.display.starts_with('-') {
            self.display.remove(0);
        } else {
            self.display.insert(0, '-');
        }
    }

    pub fn percent(&mut self) {
        if let Ok(val) = self.display.parse::<f64>() {
            self.display = format_number(val / 100.0);
        }
    }
}

impl Default for CalcApp {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_number(n: f64) -> String {
    if n.is_nan() || n.is_infinite() {
        return "Error".to_string();
    }
    if n == n.floor() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.10}", n);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_digit() {
        let mut app = CalcApp::new();
        assert_eq!(app.display, "0");

        app.input_digit('5');
        assert_eq!(app.display, "5");

        app.input_digit('3');
        assert_eq!(app.display, "53");
    }

    #[test]
    fn test_input_digit_replaces_zero() {
        let mut app = CalcApp::new();
        app.input_digit('7');
        assert_eq!(app.display, "7");
    }

    #[test]
    fn test_input_dot() {
        let mut app = CalcApp::new();
        app.input_dot();
        assert_eq!(app.display, "0.");

        app.input_digit('5');
        assert_eq!(app.display, "0.5");

        // Second dot should not be added
        app.input_dot();
        assert_eq!(app.display, "0.5");
    }

    #[test]
    fn test_addition() {
        let mut app = CalcApp::new();
        app.input_digit('2');
        app.input_operator('+');
        assert_eq!(app.first_operand, Some(2.0));
        assert_eq!(app.operator, Some('+'));

        app.input_digit('3');
        app.compute();
        assert_eq!(app.display, "5");
    }

    #[test]
    fn test_subtraction() {
        let mut app = CalcApp::new();
        app.input_digit('1');
        app.input_digit('0');
        app.input_operator('-');

        app.input_digit('3');
        app.compute();
        assert_eq!(app.display, "7");
    }

    #[test]
    fn test_multiplication() {
        let mut app = CalcApp::new();
        app.input_digit('4');
        app.input_operator('*');

        app.input_digit('5');
        app.compute();
        assert_eq!(app.display, "20");
    }

    #[test]
    fn test_division() {
        let mut app = CalcApp::new();
        app.input_digit('2');
        app.input_digit('0');
        app.input_operator('/');

        app.input_digit('4');
        app.compute();
        assert_eq!(app.display, "5");
    }

    #[test]
    fn test_division_by_zero() {
        let mut app = CalcApp::new();
        app.input_digit('1');
        app.input_operator('/');

        app.input_digit('0');
        app.compute();
        assert_eq!(app.display, "Error");
        assert_eq!(app.expression, "Cannot divide by zero");
    }

    #[test]
    fn test_clear() {
        let mut app = CalcApp::new();
        app.input_digit('5');
        app.input_operator('+');
        app.clear();

        assert_eq!(app.display, "0");
        assert_eq!(app.expression, "");
        assert_eq!(app.first_operand, None);
        assert_eq!(app.operator, None);
    }

    #[test]
    fn test_clear_entry() {
        let mut app = CalcApp::new();
        app.input_digit('5');
        app.clear_entry();
        assert_eq!(app.display, "0");
    }

    #[test]
    fn test_backspace() {
        let mut app = CalcApp::new();
        app.input_digit('5');
        app.input_digit('3');
        app.backspace();
        assert_eq!(app.display, "5");

        app.backspace();
        assert_eq!(app.display, "0");
    }

    #[test]
    fn test_toggle_sign_positive() {
        let mut app = CalcApp::new();
        app.input_digit('5');
        app.toggle_sign();
        assert_eq!(app.display, "-5");
    }

    #[test]
    fn test_toggle_sign_negative() {
        let mut app = CalcApp::new();
        app.input_digit('5');
        app.toggle_sign();
        app.toggle_sign();
        assert_eq!(app.display, "5");
    }

    #[test]
    fn test_toggle_sign_on_zero_does_nothing() {
        let mut app = CalcApp::new();
        app.toggle_sign();
        assert_eq!(app.display, "0");
    }

    #[test]
    fn test_percent() {
        let mut app = CalcApp::new();
        app.input_digit('5');
        app.input_digit('0');
        app.percent();
        assert_eq!(app.display, "0.5");
    }

    #[test]
    fn test_format_number_integer() {
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(-15.0), "-15");
    }

    #[test]
    fn test_format_number_decimal() {
        assert_eq!(format_number(3.14), "3.14");
        assert_eq!(format_number(0.5), "0.5");
    }

    #[test]
    fn test_format_number_trailing_zeros() {
        assert_eq!(format_number(1.5000), "1.5");
        assert_eq!(format_number(2.0), "2");
    }

    #[test]
    fn test_format_number_large_number() {
        assert_eq!(format_number(1000000.0), "1000000");
    }

    #[test]
    fn test_chained_operations() {
        let mut app = CalcApp::new();
        // 2 + 3 * 4 should compute left-to-right as (2 + 3) * 4 = 20
        app.input_digit('2');
        app.input_operator('+');
        app.input_digit('3');
        app.compute(); // 2 + 3 = 5
        assert_eq!(app.display, "5");

        app.input_operator('*');
        app.input_digit('4');
        app.compute(); // 5 * 4 = 20
        assert_eq!(app.display, "20");
    }
}
