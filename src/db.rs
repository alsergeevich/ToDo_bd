use rusqlite::Connection;

/// Инициализирует базу данных, создавая таблицу задач, если она не существует.
pub fn init_db(db_path: &str) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            checked INTEGER NOT NULL DEFAULT 0,
            color TEXT NOT NULL DEFAULT '#00FF00'
        )",
        (),
    )?;
    Ok(conn)
}

/// Загружает задачи из базы данных.
pub fn load_tasks(conn: &Connection) -> Result<Vec<crate::TodoItem>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, text, checked, color FROM tasks")?;
    let tasks_iter = stmt.query_map([], |row| {
        let checked_int: i32 = row.get(2)?;
        let color_hex: String = row.get(3)?;
        Ok(crate::TodoItem {
            id: row.get(0)?,
            text: row.get::<_, String>(1)?.into(),
            checked: checked_int != 0,
            color: hex_to_color(&color_hex),
        })
    })?;
    tasks_iter.collect()
}

/// Добавляет новую задачу в базу данных.
pub fn add_task(conn: &Connection, text: &str) -> Result<i32, rusqlite::Error> {
    conn.execute("INSERT INTO tasks (text) VALUES (?1)", [text])?;
    Ok(conn.last_insert_rowid() as i32)
}

/// Переключает состояние задачи (выполнена/не выполнена).
pub fn toggle_task(conn: &Connection, id: i32, checked: bool) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE tasks SET checked = ?1 WHERE id = ?2",
        rusqlite::params![checked as i32, id],
    )?;
    Ok(())
}

/// Удаляет выполненные задачи из базы данных.
pub fn delete_done(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM tasks WHERE checked = 1", ())?;
    Ok(())
}

/// Конвертирует цвет в формат HEX.
// fn color_to_hex(c: slint::Color) -> String {
//     let rgb = c.to_argb_u8();
//     format!("#{:02X}{:02X}{:02X}", rgb.red, rgb.green, rgb.blue)
// }

/// Конвертирует цвет из формата HEX в slint::Color.
fn hex_to_color(s: &str) -> slint::Color {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
    slint::Color::from_rgb_u8(r, g, b)
}
