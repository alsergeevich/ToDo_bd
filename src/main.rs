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
    {
        let tasks = db::load_tasks(&conn.borrow())?; // загружаем задачи из базы данных
        let task_model = Rc::new(slint::VecModel::from(tasks)); // создаём модель для задач
        ui.set_tasks(task_model.into()); // устанавливаем модель задач в интерфейс
    }
    
    // добавляем задачу при нажатии на кнопку
    let ui_weak_add = ui.as_weak(); // создаём слабую ссылку на интерфейс для использования в колбэке
    let conn_add = conn.clone(); // создаём копию соединения для использования в колбэке
    ui.on_add_task_m(move |text| { // колбэк для добавления задачи
        if text.is_empty() { // проверяем, что текст не пустой
            return;
        }
        let ui = ui_weak_add.unwrap(); // преобразуем слабую ссылку в сильную
        if let Ok(id) = db::add_task(&conn_add.borrow(), &text) { // если задача успешно добавлена т.е вернула id добавленной задачи. &conn_add.borrow()-результат borrow() - это ссылка на соединение, которая не владеет ресурсами, но может быть использована для чтения.
            let mut tasks: Vec<TodoItem> = ui.get_tasks().iter().collect(); // получаем текущий список задач из модели
            tasks.push(TodoItem { // добавляем новую задачу в список
                id,
                text,
                checked: false,
                color: slint::Color::from_rgb_u8(0, 255, 0),
            });
            let task_model = Rc::new(slint::VecModel::from(tasks)); // создаём модель из списка задач
            ui.set_tasks(task_model.into()); // обновляем список задач в модели
        }
    });
    
    // обновляем список задач при каждом изменении
    let ui_weak_update = ui.as_weak(); // создаём слабую ссылку на интерфейс для использования в колбэке
    let conn_update = conn.clone(); // создаём копию соединения для использования в колбэке
    ui.on_update_m(move || {
        let _ = db::delete_done(&conn_update.borrow()); // удаляем выполненные задачи в базе данных
        let ui = ui_weak_update.unwrap(); // преобразуем слабую ссылку в сильную
        if let Ok(tasks) = db::load_tasks(&conn_update.borrow()) { // загружаем задачи из базы данных, если успешно
            let task_model = Rc::new(slint::VecModel::from(tasks)); // создаём модель из списка задач
            ui.set_tasks(task_model.into()); // обновляем модель
        }
    });

    // обновляем интерфейс при изменении состояния задачи (переключение checkbox)
    let ui_weak_toggle = ui.as_weak();
    let conn_toggle = conn.clone();
    ui.on_task_toggled(move |index, checked| {
        let ui = ui_weak_toggle.unwrap();
        let mut tasks: Vec<TodoItem> = ui.get_tasks().iter().collect(); // получаем список задач
        if let Some(task) = tasks.get_mut(index as usize) { // получаем задачу по индексу если успешно
            task.checked = checked; // обновляем состояние задачи
            let _ = db::toggle_task(&conn_toggle.borrow(), task.id, checked); // обновляем состояние задачи в базе данных
        }
        let task_model = Rc::new(slint::VecModel::from(tasks)); // создаём модель из списка задач
        ui.set_tasks(task_model.into()); // обновляем модель
    });

    ui.run()?;

    Ok(())
}
