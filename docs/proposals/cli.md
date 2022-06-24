# kudoctl documentation

kudoctl is the official cli client implementation for kudo.

## Definitions

**Node :** A machine running the kudo agent.

**Workload :** A definition of a task (executable/virtual machine/container) to execute.

**Ressource :** A definition of a user or a workload, something that can be defined and sent to the kudo controller.

**Instance :** A ressource that has been applied to the kudo nodes/cluster, example : an instance of a container workload is representing the container running with the arguments of the workload definition.

## Global flags

| Name              | Shorthand | Values                             | Default        | Description                                                                       |
| ----------------- | --------- | ---------------------------------- | -------------- | --------------------------------------------------------------------------------- |
| --Version         | -V        | bool                               | false          | output the version of the client with the format : `v.major.minor.patch` (semver) |
| --help            | -h        | bool                               | false          | display the help text.                                                            |
| --config          | -c        | path                               | ~/.kudo/config | specify the path to the config file.                                              |
| --host            |           | string                             | `''`           | specify the ip of the control plane to connect to.                                |
| --verbosity-level | -v        | `'debug'/'info'/'warning'/'error'` | 2              | Set the verbosity level of the execution, see **Log format** section              |

## Exit codes

| Code | Category             | Description                                                                      |
| ---- | -------------------- | -------------------------------------------------------------------------------- |
| 0    | Success              | The command was successful.                                                      |
| 1    | Unexpected error     | This is an unhandled error and it should never happen in normal usage            |
| 2    | Syntax error         | The syntax of the command was not correct.                                       |
| 3    | Connection error     | A connectivity error or protocol error occurred.                                 |
| 4    | Server error         | An error occurred during a call to the controller.                               |
| 5    | Authentication error | An error was detected during authentication checking.                            |
| 6    | Application error    | An error occurred during processing that is performed by the client application. |

## Log format

| Severity    | Output format       |                                       |
| ----------- | ------------------- | ------------------------------------- |
| Debug       | `Debug : MESSAGE`   | Show everything                       |
| Information | `MESSAGE`           | Show interaction messages and results |
| Warning     | `Warning : MESSAGE` | Show only warning messages and errors |
| Error       | `Error : MESSAGE`   | Show only error messages              |

## Commands

Command format : `kudoctl [global options] <command> [command options]`.

---
<details> <summary><h3>get nodes</h3> </summary>

Get a list of the nodes registered to the control plane.

**Flags** :

| Name      | Shorthand | Values                                                         | Default            | Description                                                                                                                                                      |
| --------- | --------- | -------------------------------------------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| --help    | -h        |                                                                | false              | show help of the function.                                                                                                                                       |
| --format  | -F        | `"json"`, `"human_readable"`,`"yaml"`, `"delimit <character>"` | `"human_readable"` | Specifies the format of the output.                                                                                                                              |
| --verbose | -v        |                                                                | false              | Specifies whether to enable verbose mode. Use the default value of off to disable verbose mode. This option is the default value. Use on to enable verbose mode. |
| --page    | -p        |                                                                | false              | Specifies whether to display one page of text at a time or all text at one time.                                                                                 |
| --rows    | -r        | integer                                                        | 24                 | Specifies the number of rows per page to display when the **-p** parameter is on. You can specify a value in the range 1 - 100.                                  |
| --header  | -h        |                                                                | true               | Specifies whether to display the table header. Use the default value of on to display the table header. Use off to hide the table header.                        |

**Example:**

```bash
kudoctl get nodes
```

</details>

---

<details> <summary><h3>get node &lt;id&gt; </h3></summary>

Get detailed information about a node.

**Arguments** :

`id` : the id of the node

**Flags** :

| Name     | Shorthand | Values                                                          | Default            | Description                         |
| -------- | --------- | --------------------------------------------------------------- | ------------------ | ----------------------------------- |
| --help   | -h        |                                                                 | false              | show help of the function.          |
| --format | -F        | `"json"`,`"human_readable"`,  `"yaml"`, `"delimit <character>"` | `"human_readable"` | Specifies the format of the output. |

**Example:**

```bash
kudoctl get node id6875
```

</details>

---

<details> <summary><h3>get resources</h3></summary>

Get a list of the resources, the structure is described here : [resource.md](./resource.md).

**Flags** :

| Name     | Shorthand | Values                                                         | Default            | Description                                                                                                                               |
| -------- | --------- | -------------------------------------------------------------- | ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| --help   | -h        |                                                                | false              | show help of the function.                                                                                                                |
| --format | -F        | `"json"`, `"human_readable"`,`"yaml"`, `"delimit <character>"` | `"human_readable"` | Specifies the format of the output.                                                                                                       |
| --page   | -p        |                                                                | false              | Specifies whether to display one page of text at a time or all text at one time.                                                          |
| --rows   | -r        | integer                                                        | 24                 | Specifies the number of rows per page to display when the **-p** parameter is on. You can specify a value in the range 1 - 100.           |
| --header | -h        |                                                                | true               | Specifies whether to display the table header. Use the default value of on to display the table header. Use off to hide the table header. |

**Example:**

```bash
kudoctl get resources
```

</details>

---

<details> <summary><h3>get resource &lt;id&gt;</h3></summary>

This function returns the definition of a resource with the specified `id`.

**Arguments :**  

`id` : the id of the instance.

**Flags :**  

| Name     | Shorthand | Values                               | Default            | Description                                                            |
| -------- | --------- | ------------------------------------ | ------------------ | ---------------------------------------------------------------------- |
| --format |           | `"json"`,`"human_readable"`,`"yaml"` | `"human_readable"` | The output format of the resource definition,  yml is the same as yaml |
| --help   | -h        |                                      | false              | show help of the function.                                             |

**Example:**

```bash
kudoctl get resource id87967
```
  
</details>

---

<details> <summary><h3>get instances</h3></summary>

Get the list of instances and the name of the resource.

**Flags** :

| Name      | Shorthand | Values                                                         | Default            | Description                                                                                                                                                      |
| --------- | --------- | -------------------------------------------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| --help    | -h        |                                                                | false              | show help of the function.                                                                                                                                       |
| --format  | -F        | `"json"`, `"yaml"`,`"human_readable"`, `"delimit <character>"` | `"human_readable"` | Specifies the format of the output.                                                                                                                              |
| --verbose | -v        |                                                                | false              | Specifies whether to enable verbose mode. Use the default value of off to disable verbose mode. This option is the default value. Use on to enable verbose mode. |
| --page    | -p        |                                                                | false              | Specifies whether to display one page of text at a time or all text at one time.                                                                                 |
| --rows    | -r        | integer                                                        | 24                 | Specifies the number of rows per page to display when the **-p** parameter is on. You can specify a value in the range 1 - 100.                                  |
| --header  | -h        |                                                                | true               | Specifies whether to display the table header. Use the default value of on to display the table header. Use off to hide the table header.                        |

**Example:**

```bash
kudoctl get instances
```

</details>

---

<details> <summary><h3>get instance &lt;id&gt;</h3></summary>

Get details about the instance.

**Arguments** :

`id` : the id of the instance

**Flags :**  

| Name     | Shorthand | Values                               | Default            | Description                                                            |
| -------- | --------- | ------------------------------------ | ------------------ | ---------------------------------------------------------------------- |
| --format |           | `"json"`,`"human_readable"`,`"yaml"` | `"human_readable"` | The output format of the resource definition,  yml is the same as yaml |
| --help   | -h        |                                      | false              | show help of the function.                                             |

**Example:**

```bash
kudoctl get instance id9878
```

</details>
  
---

<details> <summary><h3>delete resource &lt;id&gt;</h3></summary>

Delete a resource definition and all the instances of this resource. On success the command outputs no information.

**Arguments** :

`id` : the id of the resource

**Flags :**  

| Name   | Shorthand | Values | Default | Description                |
| ------ | --------- | ------ | ------- | -------------------------- |
| --help | -h        |        | false   | show help of the function. |

**Example:**

```bash
kudoctl delete resource id8989
```

</details>

---

<details> <summary><h3>delete instance &lt;id&gt;</h3></summary>

Delete and stop an instance. On success the command outputs no information.

**Arguments** :

`id` : the id of the instance.

**Example:**

```bash
kudoctl delete instance id9898
```
  
</details>

---

<details> <summary><h3>apply</h3></summary>

Create a resource definition and instanciate it. By default if a resource with the same name exists, the resource will be updated, add the `--no-update` flag if you don’t want this behavior.

**Arguments :**

`kind` : the kind of the resource, possible values :

- workload

**Flags** :

| Name        | Shorthand | Values | Default      | Description                                             |
| ----------- | --------- | ------ | ------------ | ------------------------------------------------------- |
| --file      | -f        | Path   | `""`         | add resource definition from file.                      |
| --no-update |           | bool   | false        | If the resource already exists, don’t update it         |
| --name      |           | string | `""`         | set the name of the resource                            |
| --help      | -h        |        | false        | show help of the function.                              |
| --kind      | -k        | string | `"workload"` | set the kind of the ressource to create and instanciate |

**Kind specific flags :**

*workload :*

| Name             | Shorthand | Values   | Default       | Description                                                                           |
| ---------------- | --------- | -------- | ------------- | ------------------------------------------------------------------------------------- |
| --type           |           | string   | `"container"` | workload type                                                                         |
| --uri            |           | string   | `""`          | the uri                                                                               |
| --resources-cpu  |           | integer  | 1             | the cpu amount                                                                        |
| --resources-ram  |           | integer  | 50            | the ram amount      (MB)                                                              |
| --resources-disk |           | integer  | 1             | the disk size (GB)                                                                    |
| --port           | -p        | []string | []            | ports binding list, use multiple times to add multiple elements to the array          |
| --environment    |           | []string | []            | environment variables list, use multiple times to add multiple elements  to the array |

**Examples :**

- Add resource from file

  ```sh
  kudoctl apply -f workload.yml
  ```

</details>