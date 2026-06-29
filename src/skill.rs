use anyhow::Result;
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
    pub fn load_from(path: &Path) -> Result<Self> {
        let mut skills = Vec::new();

        if !path.exists() {
            return Ok(Self(skills));
        }

        for entry in fs::read_dir(&path)?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
        {
            let skill_md = entry.path().join("SKILL.md");
            if let Ok(content) = fs::read_to_string(&skill_md) {
                let (frontmatter, _) = markdown_frontmatter::parse::<SkillMdFrontmatter>(&content)?;
                let skill = Skill::new(&frontmatter.name, &frontmatter.description, &skill_md);
                skills.push(skill);
            }
        }

        Ok(Self(skills))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn xml(&self) -> Result<String> {
        let mut buffer = Vec::new();
        let mut writer = EmitterConfig::new()
            .write_document_declaration(false)
            .perform_indent(true)
            .create_writer(&mut buffer);

        writer.write(XmlEvent::start_element("available_skills"))?;
        for skill in self.0.iter() {
            writer.write(XmlEvent::start_element("skill"))?;
            {
                writer.write(XmlEvent::start_element("name"))?;
                writer.write(XmlEvent::characters(skill.name()))?;
                writer.write(XmlEvent::end_element())?;

                writer.write(XmlEvent::start_element("description"))?;
                writer.write(XmlEvent::characters(skill.description()))?;
                writer.write(XmlEvent::end_element())?;

                writer.write(XmlEvent::start_element("location"))?;
                writer.write(XmlEvent::characters(&skill.location().to_string_lossy()))?;
                writer.write(XmlEvent::end_element())?;
            }
            writer.write(XmlEvent::end_element())?;
        }
        writer.write(XmlEvent::end_element())?;

        Ok(String::from_utf8(buffer)?)
    }
}
