//! Построение безопасного запроса для FTS5 из «сырого» пользовательского ввода.
//!
//! ПОЧЕМУ: строку пользователя нельзя передавать в `MATCH` напрямую — спецсимволы
//! FTS5 (`"`, `*`, `:`, `(`, `-`, `AND`/`OR`/`NOT`) вызовут синтаксическую ошибку
//! или неожиданное поведение. Мы разбиваем ввод на токены, экранируем каждый как
//! строковый литерал FTS5 и добавляем префиксный поиск (`*`). Токены объединяются
//! неявным AND. См. `docs/08-SEARCH.md`.

/// Преобразует пользовательский ввод в безопасный FTS5 MATCH-запрос.
///
/// Возвращает `None`, если во вводе нет ни одного значимого символа
/// (в этом случае поиск выполнять не нужно).
///
/// Пример: `tokio async` → `"tokio"* "async"*`
pub fn to_fts_query(raw: &str) -> Option<String> {
    let tokens: Vec<String> = raw
        .split_whitespace()
        // Отбрасываем символы, не несущие смысла для полнотекстового поиска,
        // оставляя буквы/цифры/подчёркивания и внутренние дефисы слов.
        .map(sanitize_token)
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"*", t)) // строковый литерал + префиксный поиск
        .collect();

    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}

/// Оставляет в токене только буквы, цифры, `_` и `-`; кавычки экранируются
/// удвоением (на случай, если внутри остались). Возвращает очищенный токен.
fn sanitize_token(token: &str) -> String {
    let cleaned: String = token
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    // Внутри строкового литерала FTS5 двойные кавычки экранируются удвоением.
    cleaned.replace('"', "\"\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_and_wraps_tokens_with_prefix() {
        assert_eq!(
            to_fts_query("tokio async"),
            Some("\"tokio\"* \"async\"*".into())
        );
    }

    #[test]
    fn strips_fts_special_characters() {
        // Спецсимволы FTS5 не должны попадать в запрос как операторы.
        assert_eq!(to_fts_query("  (a: b*) "), Some("\"a\"* \"b\"*".into()));
    }

    #[test]
    fn empty_input_yields_none() {
        assert_eq!(to_fts_query("   "), None);
        assert_eq!(to_fts_query("***"), None);
    }

    #[test]
    fn keeps_internal_hyphen() {
        assert_eq!(to_fts_query("build-rs"), Some("\"build-rs\"*".into()));
    }
}
