# Computation offloading and energy consumption in the Internet of Things: a structured approach

## About this work and this repository
A repository containing all the necessary files to run an experimental campaign to gather power and Time to Complete data in an EDGELESS cluster. Developed for my master degree thesis (2025).

## Starting an experiment campaign to measure data for the proposed EDGELESS workflow
This file serves as a guide to reproduce the experiment runs conducted in the experiment campaigns. In this type of experiments, power consumption and Time to Complete data will be gathered for a later analysis. To exactly reproduce the experimental setup, the following main components are needed:
- **A RaspberryPI 3B+ board (RPI):** will host one $\varepsilon$-node and the device module of the Python automation script
- **A first Ubuntu virtual machine:** will host $\varepsilon$-CON, $\varepsilon$-ORC, and the Redis database used by the latter.
- **A second Ubuntu virtual machine:** will host one $\varepsilon$-node.
- **A PC:** will serve as the controller and will run the controller module of the Python automation script.
- **An Otii Arc Pro power monitor:** used to power up the RaspberryPi board.
The next steps assume that these requirements are satisfied.

### Downloading and setting up the automation scripts
A copy of the automation script must be available both on the RPI board and on the PC.
The folder will contain the automation script's code, in addition to the different EDGELESS functions code and workflow files that will be used by the framework and by the script itself. Some Python notebooks containing code to analyze the
collected data are also provided.

### Running the EDGELESS cluster
An EDGELESS cluster must be up and running in order to reproduce this type of experiments. First of all, clone the EDGELELESS repository from GitHub:
```
git clone https://github.com/edgeless-project/edgeless.git
```
Once this is done, follow the instructions provided on the EDGELESS project page to install the dependencies and build EDGELESS. Of course, in order to re-create the exact experimental setup proposed in this work, EDGELESS must be built on two virtual machines and a RaspberryPI 3B+. The following instructions assume this has been done.

On the virtual machine that will host ε-CON and ε-ORC, move into `edgeless/target/debug` and create the necessary configuration files:
```
cd edgeless/target/debug
./edgeless_cond_d -t controller.toml
./edgeless_orc_d -t orchestrator.toml
```
Once the configuration files have been created, they can be edited following the structure in the dedicated section of Appendix A of the thesis work.
<!-- Create the metrics collector ε-node:
```
./edgeless_inabox -t --metrics-collector
``` -->
The repository comes with a handy `start_cluster.sh` shell file that can be used to launch in a single command the ε-CON and the ε-ORC:
```
./start_cluster.sh
```
The commands in the script are run by prepending `RUST_LOG=info` to each instruction: this will make EDGELESS log in the command line interface the components' activity. This is rather useful to understand if everything was set up correctly. 
<!-- Once the metrics collector node starts, the ε-CON console should show a message saying that the cluster has been updated and is now composed of 1 EDGELESS node, along with the resources it is offering. --> 
On this virtual machine, a Redis in-memory database must be available, and it can be installed easily following the instructions provided in the official Redis distribution online page: the database is going to be used by the ε−ORC to mirror its internal data structures.

On the RaspberryPI, move into `edgeless/target/debug` and run:
```
./edgeless_node_d -t node.toml
```
This will create the RPI node's configuration file. The assigned UUID is random and will most certainly differ from the one shown in the file in Appendix A of the thesis.
Create the configuration file for the EDGELESS CLI tool:
```  
./edgeless_cli -t cli.toml
```
Edit the node's configuration file and, finally, launch it:
```
RUST_LOG=info ./edgeless_node_d
```
<!-- ../../../otii-automation/otii_automation/ \
    device/edgeless/rpi_node_log.log \
    2>&1 -->
Again, the ε-CON command line window should show a success message if the node has been added without any problem to the running EDGELESS cluster. 
In this case, the command line window that has been used to spawn the node will show some messages telling that the node has been successfully added to the domain.
On the second virtual machine, move into `edgeless/target/debug` and run:
```
./edgeless_node_d -t node.toml
```
Edit the node's configuration file and launch it:
```
RUST_LOG=info ./edgeless_node_d
```
<!-- RUST_LOG=info ./edgeless_node_d > vm_node_log.log 2>&1 -->
At this point, the ε-CON's console window should now show that a total of 2 nodes are available in the EDGELESS cluster. As for the RPI one, the node’s console will show some success messages, including that the node has been added to the orchestration domain.

Once the two working nodes' configuration files have been created, it is possible to inspect them to know the UUIDs that EDGELESS has automatically assigned to them. Such UUIDs must be carefully pasted into the JSON workflow file that specifies the workflow type that needs to be used during the experiment (placeholders are present in the files).

### Building the provided EDGELESS functions
In order to be executed by the nodes, the WASM bytecode of the functions composing the workflow must be available. Since these functions will run both on the RPI node and on the VM node, depending on the chosen configuration, they will need to be compiled and built on both the RaspberryPi and the virtual machine that hosts the second node.

This can be easily done by leveraging EDGELESS' CLI component, by running on both hosts the following commands:
```
cd edgeless/target/debug
./edgeless_cli function build \
    ../../../edgeless-exp-setup/functions \
    /generate\_samples/function.json
./edgeless_cli function build \
    ../../../edgeless-exp-setup/functions \
    /extract_features/function.json
./edgeless_cli function build \
    ../../../edgeless-exp-setup/functions \
    /classify/function.json
./edgeless_cli function build \
    ../../../edgeless-exp-setup/functions \
    /handle_class_result/function.json
```
These commands will create the respective `.wasm` files of the compiled functions. Such files will be placed in each function's directory inside the `functions` folder of the repository for EDGELESS to retrieve.

Errors will occur if the `.db` SQLite file containing the serialized Random Forest classifier is not placed in the right directory. The provided `edgeless_db.db` must be placed on both the RPI and the node-hosting virtual machine inside the `/var/tmp` directory, as explained in Chapter 3 of the work.

### Starting the automation script
To start the experiments, the Python automation script must first be started on the RPI.
The script can be started on the RPI board by running the following:
```
cd otii_automation
python3 main.py device
```
The program will start and wait for a UART message to be sent from the controller script.

To start the program on the controller side, run the following commands:
```  
cd otii_automation
python3 main.py controller --config <path-to-TOML-config-file>
```
An example of an experiment configuration file can be found in Appendix A of the thesis work.

The controller script will start communicating via UART to the RPI using the power monitor. Once the experiments are over, power data will be readily available in the `results` directory of the controller side automation script, while data about TtC will be found on the RPI in the locations specified in the TOML configuration file for the experiment.

### Data analysis
The folder contains the following notebooks:
- **pareto.ipynb:** can be used to produce a Pareto front graph visualization of the different configurations (x axis: average TtC data, y axis: average power data)
- **power_vs_cores.ipynb:** can be used to produce a graph showing, at the variation of the number of active cores on the RPI, the different average energy that is spent in the configurations
- **ttc_vs_cores.ipynb:** can be used to produce a graph showing, at the variation of the number of active cores on the RPI, the different average TtC that is spent in the configurations
- **ttc_analysis.ipynb:** can be used to produce grouped box plots showing the average TtC in the different offloading configurations
- **power_analysis.ipynb:** can be used to produce grouped box plots showing the average power consumption in the different offloading configurations
