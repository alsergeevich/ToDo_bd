#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
pub mod db;
use slint::Model;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Rc::new(RefCell::new(db::init_db("todo.db")?)); // создаём соединение с базой данных в виде Rc<RefCell<Connection>> для разделения соединения между колбэками 
    let ui = MainUI::new()?; // получаем интерфейс (главное окно)
    
    // загружаем задачи из базы данных и устанавливаем модель задач в интерфейс, короче блок инициализации
    let tasks = db::load_tasks(&conn.borrow())?; // загружаем задачи из базы данных
    let task_model = Rc::new(slint::VecModel::from(tasks)); // создаём модель для задач
    ui.set_tasks(task_model.clone().into()); // устанавливаем модель задач в интерфейс
    
    // добавляем задачу при нажатии на кнопку
    let task_add = task_model.clone(); // создаём копию модели для использования в колбэке
    let conn_add = conn.clone(); // создаём копию соединения для использования в колбэке
    ui.on_add_task_m(move |text| { // колбэк для добавления задачи
        if text.is_empty() { // проверяем, что текст не пустой
            return;
        }
        if let Ok(id) = db::add_task(&conn_add.borrow(), &text) { // если задача успешно добавлена т.е вернула id добавленной задачи. &conn_add.borrow()-результат borrow() - это ссылка на соединение, которая не владеет ресурсами, но может быть использована для чтения.
            task_add.push(TodoItem { // добавляем новую задачу в существующую модель
                id,
                text,
                checked: false,
                color: slint::Color::from_rgb_u8(0, 255, 0),
            });
        }
    });
    
    // обновляем список задач при каждом изменении
    let task_update = task_model.clone(); // создаём копию модели для использования в колбэке
    let conn_update = conn.clone(); // создаём копию соединения для использования в колбэке
    ui.on_update_m(move || {
        let _ = db::delete_done(&conn_update.borrow()); // удаляем выполненные задачи в базе данных
        if let Ok(tasks) = db::load_tasks(&conn_update.borrow()) { // загружаем задачи из базы данных, если успешно
            task_update.set_vec(tasks); // обновляем существующую модель
        }
    });

    // обновляем интерфейс при изменении состояния задачи (переключение checkbox)
    let task_model_toggle = task_model.clone(); // создаём копию модели для использования в колбэке
    let conn_toggle = conn.clone(); // создаём копию соединения для использования в колбэке
    ui.on_task_toggled(move |index, checked| {
        
        if let Some(mut task) = task_model_toggle.row_data(index as usize) { // получаем задачу по индексу если успешно
            task.checked = checked; // обновляем состояние задачи
            let _ = db::toggle_task(&conn_toggle.borrow(), task.id, checked); // обновляем состояние задачи в базе данных
            task_model_toggle.set_row_data(index as usize, task); // обновляем модель задачи
        }
        
    });

    ui.run()?;

    Ok(())
}
