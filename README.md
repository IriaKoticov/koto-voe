---

# Koto-voe 🐾

**Koto-voe** — это веб-приложение, в котором пользователи могут свайпать карточки с котиками, добавлять любимых котов в личный альбом, а также просматривать топ самых популярных котиков по оценкам других пользователей.

### Основные возможности:


* Свайп-интерфейс с анимацией для выбора любимых котиков
* Личный альбом с возможностью удаления карточек
* Страница топа с тремя самыми оценёнными котами
* Хранение изображений в PostgreSQL (в формате BYTEA)
* Хранимая функция для расчёта средней оценки кота

### Технологии:

* **Frontend:** Dioxus (Rust)
* **Backend:** Axum (Rust), SQLx
* **База данных:** PostgreSQL

