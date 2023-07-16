use leetcode_tui_rs::app_ui::list::StatefulList;
use leetcode_tui_rs::config::{self, Config};
use leetcode_tui_rs::db_ops::ModelUtils;
use leetcode_tui_rs::deserializers::question::{ProblemSetQuestionListQuery, Question};
use leetcode_tui_rs::entities::QuestionModel;
use leetcode_tui_rs::graphql::problemset_question_list::Query;
use leetcode_tui_rs::graphql::GQLLeetcodeQuery;
use reqwest::header::{HeaderMap, HeaderValue};
use sea_orm::Database;
use tracing;
use tracing_subscriber;

use leetcode_tui_rs::app_ui::app::{App, AppResult, TTReciever, Widget};
use leetcode_tui_rs::app_ui::event::{Event, EventHandler};
use leetcode_tui_rs::app_ui::handler::handle_key_events;
use leetcode_tui_rs::app_ui::tui::Tui;
use leetcode_tui_rs::entities::topic_tag::Model as TopicTagModel;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::HashMap;
use std::io;

use once_cell::sync::Lazy;

static CONFIG: Lazy<config::Config> = Lazy::new(|| Config::from_file("./leetcode.config"));

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    let csrf = CONFIG.leetcode.csrftoken.as_str();
    let sess = CONFIG.leetcode.leetcode_session.as_str();
    let mut headers = HeaderMap::new();
    headers.append(
        "Cookie",
        HeaderValue::from_str(&format!("LEETCODE_SESSION={sess}; csrftoken={csrf}")).unwrap(),
    );
    reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap()
});

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let database_client = Database::connect(CONFIG.db.url.as_str()).await.unwrap();

    // let query = Query::default();
    // let query_response: ProblemSetQuestionListQuery = query.post(&CLIENT).await;
    // Question::multi_insert(&database_client, query_response.get_questions()).await;

    // Create an application.
    use crossbeam;

    let (send, recv) = crossbeam::channel::unbounded();

    let mut q =
        leetcode_tui_rs::db_ops::topic_tag::query::get_questions_by_topic(&database_client, "")
            .await;

    while !q.is_empty() {
        let qp = q.pop();
        if let Some(qp) = qp {
            send.send(qp).unwrap();
        };
    }

    tokio::task::spawn_blocking(|| run_app(recv).unwrap());

    Ok(())
}

fn run_app(recv: TTReciever) -> AppResult<()> {
    let mut ql: HashMap<String, Vec<QuestionModel>> = HashMap::new();
    let mut topic_tags = vec![];

    while let Ok((topic_tag, mut questions)) = recv.recv() {
        if let Some(name) = &topic_tag.name {
            ql.entry(name.clone())
                .or_insert(vec![])
                .append(&mut questions);
        }
        topic_tags.push(topic_tag);
    }

    let questions = vec![];

    let mut qm: StatefulList<QuestionModel> = StatefulList::with_items(questions);
    let mut ttm: StatefulList<TopicTagModel> = StatefulList::with_items(topic_tags);
    let question_stateful = Widget::QuestionList(&mut qm);
    let topic_tag_stateful = Widget::TopicTagList(&mut ttm);
    let mut vw = vec![topic_tag_stateful, question_stateful];

    let mut app = App::new(&mut vw, &ql);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(50);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
