use super::Resource;

pub fn parseYAMLResource(content: &str) -> Result<Resource, serde_yaml::Error> {
    let resource: Resource = serde_yaml::from_str(content)?;
    Ok(resource)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse_yaml_resource() {
        let content = r#"
kind: workload
name: my-workload
uri: my-workload-uri
resources:
  cpu: 1
  memory: 2
  disk: 3
"#;
        let resource = parseYAMLResource(content).unwrap();

        match resource {
            Resource::Workload(workload) => {
                assert_eq!(workload.name, "my-workload");
                assert_eq!(workload.uri, "my-workload-uri");
                assert_eq!(workload.resources.cpu, 1);
                assert_eq!(workload.resources.memory, 2);
                assert_eq!(workload.resources.disk, 3);
            }
            _ => panic!("Unexpected resource type"),
        }
    }

    #[test]
    fn test_parse_yaml_resource_with_ports() {
        let content = r#"
kind: workload
name: my-workload
uri: my-workload-uri
resources:
  cpu: 1
  memory: 2
  disk: 3
ports:
  - "8080:8080"
  - "8081:8081"
"#;
        let resource = parseYAMLResource(content).unwrap();

        match resource {
            Resource::Workload(workload) => {
                assert_eq!(workload.ports.as_ref().unwrap().len(), 2);
                assert_eq!(workload.ports.as_ref().unwrap()[0], "8080:8080");
                assert_eq!(workload.ports.as_ref().unwrap()[1], "8081:8081");
            }
            _ => panic!("Unexpected resource type"),
        }
    }

    #[test]
    fn test_parse_yaml_resource_with_env() {
        let content = r#"
kind: workload
name: my-workload
uri: my-workload-uri
resources:
  cpu: 1
  memory: 2
  disk: 3
env:
    - "KEY1=VALUE1"
    - "KEY2=VALUE2"
"#;
        let resource = parseYAMLResource(content).unwrap();

        match resource {
            Resource::Workload(workload) => {
                assert_eq!(workload.enviroment.as_ref().unwrap().len(), 2);
                assert_eq!(workload.enviroment.as_ref().unwrap()[0], "KEY1=VALUE1");
                assert_eq!(workload.enviroment.as_ref().unwrap()[1], "KEY2=VALUE2");
            }
            _ => panic!("Unexpected resource type"),
        }
    }

    #[test]
    fn test_parse_incomplete_yaml_resource(){
        let content = r#"
kind: workload
name: my-workload
resources:
    cpu: 1
    memory: 2
    disk: 3
"#;
        let resource = parseYAMLResource(content);
        assert!(resource.is_err());
    }
}
