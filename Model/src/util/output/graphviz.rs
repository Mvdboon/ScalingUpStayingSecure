use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use log::trace;

use crate::model::MyGraph;
use crate::util::UtilError;

/// A helper function to write a string to a file
pub fn write_display_string_to_file<T>(filename: &T, dot: impl Display) -> Result<(), UtilError>
where
    T: AsRef<Path> + Display,
{
    match File::create(filename) {
        Ok(mut file) => Ok(file.write_all(format!("{dot}").as_bytes())?),
        Err(e) => Err(e.into()),
    }
}

/// A helper function that runs the Graphviz command to convert dot file to png.
/// It uses the Circo algorithm.
pub fn run_graphviz<T>(dot_file: T) -> Result<(), UtilError>
where
    T: AsRef<Path> + Display,
{
    let png_output = dot_file.as_ref().display().to_string().replace(".dot", ".png");
    let output = Command::new("circo.exe")
        .args([
            format!("{}", dot_file.as_ref().display()),
            "-Tpng".to_string(),
            format!("-o{png_output}",),
        ])
        .output()?;
    trace!("{:?}", output.stdout);
    Ok(())
}

/// A helper function that creates the dot file and calls the
/// [`write_display_string_to_file`] function.
pub fn write_dot_file<T>(graph: &MyGraph, filename: &T) -> Result<(), UtilError>
where
    T: AsRef<Path> + Display,
{
    write_display_string_to_file(filename, graph.get_dot_string())
}

/// Print the graph in png and dotfile. Provide the model and filename of the
/// dotfile including .dot extension.
pub fn output_graph_to_png<T>(filename: &T, model: &MyGraph) -> Result<(), UtilError>
where
    T: AsRef<Path> + Display,
{
    write_dot_file(model, filename)?;
    run_graphviz(filename)
}

#[cfg(test)]
mod test_io {
    use super::*;

    #[test]
    fn test_write_display_string_to_file_util_error() {
        let graph = MyGraph::default();
        let x = write_display_string_to_file(&"".to_string(), graph.get_dot_string()).expect_err("msg");
        dbg!(x);
    }
}
