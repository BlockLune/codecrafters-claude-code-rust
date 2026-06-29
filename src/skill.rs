use std::fs;
use std::path::{Path, PathBuf};
use xml::writer::{EmitterConfig, XmlEvent};

#[derive(serde::Deserialize, Debug)]
struct SkillMdFrontmatter {
    name: String,
    description: String,
}

pub struct Skill {
    name: String,
    description: String,
    location: PathBuf,
}

impl Skill {
    pub fn new(name: &str, description: &str, location: &Path) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            location: location.to_path_buf(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn location(&self) -> &Path {
        &self.location
    }
}

pub struct LoadedSkills(Vec<Skill>);

impl LoadedSkills {
    pub fn load_from(path: &Path) -> Self {
        let mut skills = Vec::new();

        if !path.exists() {
            return Self(skills);
        }

        let entries = fs::read_dir(&path)
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        for entry in entries {
            let skill_md = entry.path().join("SKILL.md");
            if let Ok(content) = fs::read_to_string(&skill_md) {
                let (frontmatter, _) =
                    markdown_frontmatter::parse::<SkillMdFrontmatter>(&content).unwrap();
                let skill = Skill::new(&frontmatter.name, &frontmatter.description, &skill_md);
                skills.push(skill);
            }
        }

        Self(skills)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn xml(&self) -> String {
        let mut buffer = Vec::new();
        let mut writer = EmitterConfig::new()
            .write_document_declaration(false)
            .perform_indent(true)
            .create_writer(&mut buffer);

        writer
            .write(XmlEvent::start_element("available_skills"))
            .unwrap();

        for skill in self.0.iter() {
            writer.write(XmlEvent::start_element("skill")).unwrap();

            {
                writer.write(XmlEvent::start_element("name")).unwrap();
                writer.write(XmlEvent::characters(skill.name())).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();

                writer
                    .write(XmlEvent::start_element("description"))
                    .unwrap();
                writer
                    .write(XmlEvent::characters(skill.description()))
                    .unwrap();
                writer.write(XmlEvent::end_element()).unwrap();

                writer.write(XmlEvent::start_element("location")).unwrap();
                writer
                    .write(XmlEvent::characters(&skill.location().to_string_lossy()))
                    .unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            }

            writer.write(XmlEvent::end_element()).unwrap();
        }

        writer.write(XmlEvent::end_element()).unwrap();

        String::from_utf8(buffer).unwrap()
    }
}
