# Workload schema

## name

Name of the workload, should be unique in the controller’s database.

## type

Type of the workload :

- `"container"`
- `"vm"`
- `"binary"`

## executable

<!-- maybe change name -->

The name of the container, or an URI to the vm image or binary executable.

## ressources

Definition of the maximum (container, binary) or allocated (vm) ressources.

### ressources.cpu

CPU power, unit is to be defined.

### ressources.ram

Memory amount, unit is MB.

### ressources.disk

Disk size, unit is GB.

## ports

List of port mapping with the format `"external:internal"`.
