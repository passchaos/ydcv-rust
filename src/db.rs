use anyhow::bail;
use anyhow::Result;
use redb::{Database, ReadableTable, TableDefinition};
use serde_derive::{Deserialize, Serialize};
const TABLE: TableDefinition<&str, &str> = TableDefinition::new("ydcv");
fn get_db() -> Result<Database> {
    let Some(mut dir) = dirs::home_dir() else {
        bail!("no home dir found");
    };

    dir.push("proliferation/english");

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    dir.push("ydcv.redb");

    let db = Database::create(dir)?;

    Ok(db)
}

fn get_value(key: &str) -> Result<Option<String>> {
    let db = get_db()?;

    let read_txn = db.begin_read()?;
    let table = match read_txn.open_table(TABLE) {
        Ok(t) => t,
        Err(e) => match e {
            redb::TableError::TableDoesNotExist(msg) => {
                eprintln!("table: {msg} not exist");
                return Ok(None);
            }
            _ => return Err(e.into()),
        },
    };

    let v = table.get(key)?;

    Ok(v.map(|s| s.value().to_string()))
}

fn insert_value(key: &str, value: &str) -> Result<()> {
    let db = get_db()?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(key, value)?;
    }
    write_txn.commit()?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Answer {
    pub explain: String,
    pub query_count: u64,
}

pub fn get_saved_answer(query: &str) -> Result<Option<Answer>> {
    let saved_answer = get_value(query)?;

    let res = if let Some(answer) = saved_answer {
        let answer: Answer = serde_json::from_str(&answer)?;

        Some(answer)
    } else {
        None
    };

    Ok(res)
}

pub fn step_forward_with_web_result(query: &str, explain: String) -> Result<Answer> {
    let saved_answer = get_saved_answer(query)?;

    let (explain, query_count) = match saved_answer {
        Some(a) => (explain, a.query_count + 1),
        None => (explain, 1),
    };

    let new_answer = Answer {
        explain,
        query_count,
    };

    insert_value(query, &serde_json::to_string(&new_answer)?)?;

    Ok(new_answer)
}

pub fn step_forward_with_local_only(query: &str) -> Result<Option<Answer>> {
    let saved_answer = get_saved_answer(query)?;

    let (explain, query_count) = match saved_answer {
        Some(a) => (a.explain, a.query_count + 1),
        None => {
            return Ok(None);
        }
    };

    let new_answer = Answer {
        explain,
        query_count,
    };

    insert_value(query, &serde_json::to_string(&new_answer)?)?;

    Ok(Some(new_answer))
}
