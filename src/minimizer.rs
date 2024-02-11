use std::{
    io::{BufRead, Read, Write},
    path::Path,
};

pub struct RustSourceFile {
    lines: Vec<String>,
    is_removed: Vec<bool>,
}
impl RustSourceFile {
    pub fn from_file(mut src: impl Read + BufRead) -> std::io::Result<Self> {
        let mut lines = Vec::new();
        let mut is_removed = Vec::new();
        for line in src.lines() {
            lines.push(line?);
            is_removed.push(false);
        }
        Ok(Self { lines, is_removed })
    }
    pub fn to_string(&self) -> String {
        self.lines().cloned().collect()
    }
    fn remove_line(&mut self, line: usize) {
        self.is_removed[line] = true;
    }
    fn restore_line(&mut self, line: usize) {
        self.is_removed[line] = false;
    }
    fn try_remove_line(
        &mut self,
        line: usize,
        is_equivalent: &impl Fn(&Self) -> Result<(), String>,
        last_ok_path: &Path,
    ) {
        if self.is_removed[line] {
            return;
        }
        self.remove_line(line);
        let can_remove = match is_equivalent(self) {
            Ok(_) => true,
            Err(err) => {
                println!(
                    "Can't remove line {line} becasue {err}.",
                    line = line + 1,
                    err = &err[..err.len().min(90)]
                );
                false
            }
        };
        if !can_remove {
            self.restore_line(line);
        } else {
            println!("Removing line:{line}", line = line + 1);
            self.into_file(std::fs::File::create(last_ok_path).unwrap())
                .unwrap();
        }
    }
    fn try_remove_line_span(
        &mut self,
        span: std::ops::Range<usize>,
        is_equivalent: &impl Fn(&Self) -> Result<(), String>,
        last_ok_path: &Path,
    ) -> Result<(), String> {
        if self.is_removed[span.clone()]
            .iter()
            .all(|is_removed| *is_removed)
        {
            return Ok(());
        }
        let saved: Box<[bool]> = self.is_removed[span.clone()].into();

        for line in span.clone() {
            self.remove_line(line);
        }
        let is_equivalent = is_equivalent(self);
        let can_remove = match &is_equivalent {
            Ok(_) => true,
            Err(err) => {
                println!(
                    "Can't remove span {span:?} becasue {err}.",
                    err = &err[..err.len().min(90)]
                );
                false
            }
        };
        if !can_remove {
            for (line, was_removed) in span.zip(saved.iter()) {
                if !was_removed {
                    self.restore_line(line)
                };
            }
        } else {
            println!("Removing span:{span:?}");
            self.into_file(std::fs::File::create(last_ok_path).unwrap())
                .unwrap();
        }
        is_equivalent
    }
    pub fn removed_lines(&self) -> usize {
        self.is_removed
            .iter()
            .filter(|is_removed| **is_removed)
            .count()
    }
    pub fn into_file(&self, mut w: impl Write) -> std::io::Result<()> {
        for (line, is_removed) in self.lines.iter().zip(self.is_removed.iter()) {
            if !is_removed {
                writeln!(w, "{line}")?;
            }
        }
        Ok(())
    }
    pub fn try_remove_lines(
        &mut self,
        is_equivalent: &impl Fn(&Self) -> Result<(), String>,
        last_ok_path: &Path,
    ) {
        if let Err(err) = is_equivalent(self) {
            eprintln!(
                "Could not mnimize because the original contained errors {err}.",
                err = &err[..err.len().min(1_000_000)]
            );
            panic!();
        }
        let line_count = self.lines.len();
        // For time estimates
        let start = std::time::Instant::now();
        for index in 0..(line_count - 5) {
            if let Err(_) =
                        self.try_remove_line_span(index..(index + 3), is_equivalent, last_ok_path)
                    {
                        if let Err(_) = self.try_remove_line_span(
                            index..(index + 2),
                            is_equivalent,
                            last_ok_path,
                        ) {
                            self.try_remove_line(index, is_equivalent, last_ok_path);
                        }
                    }
            let time_per_line = (start.elapsed().as_millis() as f64 / 1000.0) / (index as f64);
            let estimate_sec = time_per_line * (line_count - index) as f64;
            let expected_minmization = (self.removed_lines() as f64 / index as f64) * 100.0;
            println!("Trying to remove line {index}. Progress:{:.2}% tpl:{time_per_line:.2}s. Remaining {estimate_sec:.2}s expected minimization:{expected_minmization:.2}%",(index as f64/line_count as f64)*100.0,index = index + 1);
        }
    }
    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines
            .iter()
            .zip(self.is_removed.iter())
            .filter_map(|(line, is_removed)| if *is_removed { None } else { Some(line) })
    }
}
