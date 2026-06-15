# Terrafier

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Work in Progress](https://img.shields.io/badge/status-work--in--progress-yellow?style=for-the-badge)
![License](https://img.shields.io/badge/license-GPL--3.0-blue?style=for-the-badge)

> Rust-native Minecraft world painter - быстрее, легче, современнее.

**Terrafier** - нативная замена [WorldPainter](https://www.worldpainter.net) на Rust.
Рендеринг карт Minecraft через GPU, параллельный экспорт, CLI-first архитектура.

## Быстрый старт

```bash
# Установка
cargo install terrafier-cli

# Создать новый мир
terrafier new my_world --seed 12345

# Экспортировать в Minecraft
terrafier export my_world --output ./minecraft-worlds

# Посмотреть информацию
terrafier info my_world

# Рендер превью
terrafier render my_world --output preview.png
```

## Возможности

- **GPU-рендеринг** - плавный просмотр карты 30-60 FPS
- **Параллельный экспорт** - в 3-5 раз быстрее Java-аналога
- **Компактный бинарник** - 15-25 MB, без JRE
- **CLI + GUI** - и для скриптов, и для интерактивной работы
- **Совместимость** - читает .world файлы WorldPainter, экспортирует Anvil 1.21+

## Структура

```
core/   - библиотека (модель мира, парсинг, экспорт)
cli/    - CLI-инструмент
gui/    - GUI-приложение на egui + wgpu
```

## Лицензия

GNU General Public License v3.0
