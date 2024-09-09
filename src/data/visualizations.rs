//! # Visualization Module
//!
//! This module handles generating the Visualizations using the Python backend.

use std::{collections::HashMap, env, path::PathBuf};
use thiserror::Error;
use pyembed::MainPythonInterpreter;

include!("../../pyembedded/default_python_config.rs");

/// The error types for the visualization module.
#[derive(Error, Debug)]
pub enum VisualizationError {
    /// Occurs during an environment error.
    #[error("Environment error: {0}")]
    EnvError(String),

    /// Occurs when invoking the Python interpreter fails.
    #[error("Python interpreter error: {0}")]
    InterpreterError(String),

    /// Occurs when the Python backend fails to complete an operation.
    #[error("Python Error: {0}")]
    PythonError(String),
}

/// Enum to represent which section each visualization corresponds to.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ReportSection {
    /// The missing values analysis section.
    MissingValues,
}

/// Struct which holds the filepaths to the generated visualizations.
#[derive(Debug)]
pub struct VisualizationManager {
    /// Outter hashmap represents the report section that the visualizations correspond to and the
    /// inner hashmap holds the visualization's title and path to the image file.
    pub visualizations: HashMap<ReportSection, HashMap<String, PathBuf>>,
}

impl VisualizationManager {
    /// Constructor for the VisualizationManager struct.
    ///
    /// ### Parameters
    ///
    /// - `path`: Path to the data file.
    pub fn new(
        path: &PathBuf,
        file_type: &str,
        output_path: &PathBuf,
    ) -> Result<Self, VisualizationError> {
        let config = default_python_config();

        // Add the python directory to the Python path.
        let current_dir = env::current_dir().map_err(|e| VisualizationError::EnvError(e.to_string()))?;
        let python_path = current_dir.join("py");
        println!("{}", python_path.display());
        std::env::set_var("PYTHONPATH", python_path.to_str().unwrap());

        let interpreter = MainPythonInterpreter::new(config).map_err(|e| VisualizationError::InterpreterError(e.to_string()))?;

        let result = interpreter.with_gil(|py| {
            // TODO : debugging
            let sys = py.import("sys").unwrap();
            let sys_path: Vec<String> = sys.getattr("path").unwrap().extract().unwrap();
            println!("Python sys.path: {:?}", sys_path);
            let os = py.import("os").unwrap();
            let listdir: Vec<String> = os.call_method1("listdir", ("py",)).unwrap().extract().unwrap();
            println!("Contents of /home/seank/projects/personal/leads/py: {:?}", listdir);
            let pckgutil = py.import("pkgutil").unwrap();
            let pkg_exists = pckgutil.call_method1("find_loader", ("viz_lib",)).unwrap();
            if pkg_exists.is_none() {
                println!("viz_lib not found");
            } else {
                println!("FINALLY aksdjf asdkjf ;kfjd")
            }

            let main_module = py.import("viz_lib").map_err(|e| {
                VisualizationError::PythonError(format!("Unable to import viz_lib: {}", e))
            })?;
            let function = main_module
                .getattr("generate_visualizations")
                .map_err(|e| {
                    VisualizationError::PythonError(format!(
                        "Unable to import function `generate_visualizations`: {}",
                        e
                    ))
                })?;
            let result: HashMap<String, String> = function
                .call1((path.to_str(), file_type, output_path.to_str()))
                .and_then(|v| v.extract())
                .map_err(|e| VisualizationError::PythonError(format!("{:?}", e)))?;
            Ok(result)
        })?;

        let mut visualizations = HashMap::new();
        let mut missing_values_map = HashMap::new();

        for (title, path) in result {
            missing_values_map.insert(title, PathBuf::from(path));
        }
        visualizations.insert(ReportSection::MissingValues, missing_values_map);

        let visualization_manager = Self {visualizations};
        println!("{:?}", visualization_manager);

        Ok(visualization_manager)
    }
}
