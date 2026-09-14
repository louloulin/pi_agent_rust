//! Session integration for the pure `pi-todo-core` task-list state machine.
use crate::error::{Error, Result};
pub use pi_todo_core::{apply_op, TodoError, TodoInitPhase, TodoList, TodoOp, TodoPhase, TodoStatus, TodoTask};
use serde::Deserialize;
pub const TODO_LIST_SCHEMA: &str = "todo_list.v1";
#[must_use]
pub fn latest_from_entries<'a, I>(entries: I) -> TodoList where I: IntoIterator<Item=&'a crate::session::SessionEntry> { let mut latest=TodoList::default(); for entry in entries { if let crate::session::SessionEntry::Custom(custom)=entry && custom.custom_type==TODO_LIST_SCHEMA && let Some(data)=&custom.data && let Ok(list)=TodoList::deserialize(data){latest=list;} } latest }
pub struct TodoTool { session: std::sync::Arc<asupersync::sync::Mutex<crate::session::Session>> }
impl TodoTool { pub const fn new(session:std::sync::Arc<asupersync::sync::Mutex<crate::session::Session>>)->Self{Self{session}} }
#[async_trait::async_trait]
impl crate::tools::Tool for TodoTool {
 fn name(&self)->&str{"todo"} fn label(&self)->&str{"Todo"}
 fn description(&self)->&str{"Maintain the session task list. Ops: init, start, done, drop, block, unblock, rm, append, view. Tasks use exact content and completed tasks never revert."}
 fn parameters(&self)->serde_json::Value{serde_json::json!({"type":"object","properties":{"op":{"type":"string","enum":["init","start","done","drop","block","unblock","rm","append","view"]},"phases":{"type":"array"},"tasks":{"type":"array"},"task":{"type":"string"},"reason":{"type":"string"},"phase":{"type":"string"}},"required":["op"],"additionalProperties":false})}
 fn effects(&self)->crate::tools::ToolEffects{crate::tools::ToolEffects::read()}
 async fn execute(&self,_:&str,input:serde_json::Value,_:Option<Box<dyn Fn(crate::tools::ToolUpdate)+Send+Sync>>)->Result<crate::tools::ToolOutput>{ let op:TodoOp=serde_json::from_value(input).map_err(|e|Error::validation(format!("Invalid todo input: {e}")))?; let cx=crate::agent_cx::AgentCx::for_current_or_request(); let mut session=self.session.lock(cx.cx()).await.map_err(|e|Error::session(format!("Failed to lock session: {e}")))?; let mut list=latest_from_entries(session.entries_for_current_path()); let mutated=apply_op(&mut list,&op).map_err(|e|Error::validation(e.to_string()))?; let state=serde_json::to_value(&list).map_err(|e|Error::session(format!("todo state serialize: {e}")))?; if mutated {session.append_custom_entry(TODO_LIST_SCHEMA.to_string(),Some(state.clone()));} drop(session); Ok(crate::tools::ToolOutput{content:vec![crate::model::ContentBlock::Text(crate::model::TextContent::new(list.render()))],details:Some(serde_json::json!({"schema":TODO_LIST_SCHEMA,"list":state,"summary":list.summary_line()})),is_error:false}) }
}
