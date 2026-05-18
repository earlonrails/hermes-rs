use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::path::Path;
use tracing::{debug, info};
use uuid::Uuid;

use crate::store::SkillStore;
use crate::Skill;

pub struct SkillManager {
    store: SkillStore,
    embed_model: TextEmbedding,
}

impl SkillManager {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, anyhow::Error> {
        let store = SkillStore::new(db_path)?;
        
        info!("Initializing local embedding model (this may download model weights the first time)...");
        let embed_model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::AllMiniLML6V2,
            show_download_progress: true,
            ..Default::default()
        })?;
        
        Ok(Self {
            store,
            embed_model,
        })
    }

    pub fn create_skill(&self, name: &str, description: &str, instructions: &str) -> Result<Skill, anyhow::Error> {
        let text_to_embed = format!("{} {}", description, instructions);
        
        // Generate embedding
        let embeddings = self.embed_model.embed(vec![text_to_embed], None)?;
        let embedding = &embeddings[0]; // fastembed returns Vec<f32>
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs() as i64;
            
        let skill = Skill {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            instructions: instructions.to_string(),
            created_at: now,
            updated_at: now,
        };
        
        self.store.insert_skill(&skill, embedding)?;
        debug!("Created skill {}", skill.id);
        
        Ok(skill)
    }

    /// Retrieve the top K most relevant skills for a given query context
    pub fn search_skills(&self, query: &str, top_k: usize) -> Result<Vec<Skill>, anyhow::Error> {
        let query_embeddings = self.embed_model.embed(vec![query.to_string()], None)?;
        let query_vec = &query_embeddings[0];
        
        let all_embeddings = self.store.get_all_embeddings()?;
        
        let mut scored_skills: Vec<(f32, String)> = all_embeddings.into_iter().map(|(id, embed)| {
            let score = Self::cosine_similarity(query_vec, &embed);
            (score, id)
        }).collect();
        
        // Sort descending by score
        scored_skills.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        let top_ids = scored_skills.into_iter().take(top_k).collect::<Vec<_>>();
        
        let mut results = Vec::new();
        for (_, id) in top_ids {
            if let Some(skill) = self.store.get_skill(&id)? {
                results.push(skill);
            }
        }
        
        Ok(results)
    }
    
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        
        for i in 0..a.len() {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a.sqrt() * norm_b.sqrt())
        }
    }
}
