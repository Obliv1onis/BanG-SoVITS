use serde::{Deserialize, Serialize};
use std::{fs, fs::OpenOptions, net::TcpListener, path::{Path, PathBuf}, process::Stdio, sync::Mutex, time::{Duration, SystemTime, UNIX_EPOCH}};
use tauri::{Manager, State};
use tokio::{process::{Child, Command}, time::sleep};
use walkdir::WalkDir;

#[derive(Default)] struct Engine { child:Mutex<Option<Child>>, port:Mutex<Option<u16>>, current:Mutex<Option<(String,String)>> }
#[derive(Serialize, Clone)] #[serde(rename_all="camelCase")] struct Reference { path:String, text:String }
#[derive(Serialize, Clone)] #[serde(rename_all="camelCase")] struct Character { id:String, name:String, band:String, variant:Option<String>, gpt_weight:Option<String>, sovits_weight:Option<String>, references:Vec<Reference> }
#[derive(Serialize)] #[serde(rename_all="camelCase")] struct Catalog { characters:Vec<Character>, root:String, engine_root:String }
#[derive(Serialize)] #[serde(rename_all="camelCase")] struct HistoryItem { path:String, name:String, created:u64, size:u64 }
#[derive(Deserialize)] #[serde(rename_all="camelCase")] struct GenerateRequest { character_id:String, text:String, text_lang:String, reference_path:String, prompt_text:String, speed:f32, temperature:f32, seed:i64 }
fn http_client()->Result<reqwest::Client,String>{reqwest::Client::builder().no_proxy().build().map_err(|e|format!("无法创建本地 HTTP 客户端：{e}"))}

fn project_root() -> Result<PathBuf,String> {
  let cwd=std::env::current_dir().map_err(|e|e.to_string())?;
  let built_from=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
  let bundled=std::env::current_exe().ok().and_then(|p|p.parent()?.parent().map(|p|p.join("Resources"))).unwrap_or_default();
  for p in [bundled,cwd.clone(),cwd.join(".."),cwd.join("../.."),built_from] { if p.join("角色语音").is_dir() && p.join("engine").join("api_v2.py").is_file(){return fs::canonicalize(p).map_err(|e|e.to_string())} }
  Err("找不到角色语音与 GPT-SoVITS 目录。请从项目目录启动应用。".into())
}
fn app_data_root()->Result<PathBuf,String>{
 let home=std::env::var_os("HOME").map(PathBuf::from).ok_or("找不到用户目录")?;
 let path=home.join("Library/Application Support/studio.bangdream.voice");fs::create_dir_all(&path).map_err(|e|format!("无法创建应用数据目录：{e}"))?;Ok(path)
}
fn output_root()->Result<PathBuf,String>{let path=app_data_root()?.join("output");fs::create_dir_all(&path).map_err(|e|format!("无法创建生成记录目录：{e}"))?;Ok(path)}
fn stem(path:&Path)->String{path.file_stem().unwrap_or_default().to_string_lossy().to_string()}
fn key(s:&str)->String{s.to_lowercase().replace([' ','_','-','（','）','(',')','月','ノ','之','森','限','定','夹','ヶ'],"").replace("燈","灯").replace("愛","爱").replace("長","长").replace("樂","乐").replace("葉","叶").replace(['豐','豊'],"丰").replace("麥","麦").replace(['戶','戸'],"户").replace(['澤','沢'],"泽").replace("彌","弥").replace("綾","绫").replace("紗","纱").replace("亞","亚").replace("奧","奥").replace("瀨","濑").replace("緋","绯").replace("瑪","玛").replace("麗","丽").replace("鶫","鸫").replace("華","华").replace("鈴","铃").replace("鷺","鹭").replace("聖","圣").replace("宮","宫").replace("園","园").replace("湊","凑").replace("蘭","兰").replace("倉","仓").replace("廣","广")}
fn model_name(name:&str)->&str{match name{
 "ksm"|"戶山香澄"|"户山香澄"=>"戸山香澄","上原緋瑪麗"|"上原绯玛丽"=>"上原ひまり","羽澤鶇"|"羽泽鸫"=>"羽沢鶫","青葉摩卡"|"青叶摩卡"=>"青葉モカ",
 "佐藤益木"=>"MASKING","和奏瑞依"=>"LAYER","朝日六花"=>"LOCK","珠手知由"=>"Chu²","鳰原令王那"=>"PAREO",
 "墨提斯"=>"Mortis","梦中情祥"=>"豊川祥子",_=>name}}
fn choose_weight(files:&[PathBuf],name:&str,folder:&str)->Option<String>{
 let ck=key(model_name(name));let mut found=files.iter().filter(|p|key(&stem(p)).contains(&ck)).collect::<Vec<_>>();
 found.sort_by_key(|p|stem(p));
 let preferred=if folder.contains("月ノ森")||folder.contains("月之森"){"白"}else{"黒"};
 found.iter().find(|p|stem(p).contains(preferred)).or_else(||found.first()).map(|p|p.to_string_lossy().to_string())
}
#[tauri::command]
fn load_catalog()->Result<Catalog,String>{
 let root=project_root()?;let voices=root.join("角色语音");let engine=root.join("engine");let gdir=engine.join("GPT_weights_v2ProPlus");let sdir=engine.join("SoVITS_weights_v2ProPlus");
 let gp=fs::read_dir(&gdir).map_err(|e|e.to_string())?.flatten().map(|e|e.path()).filter(|p|p.extension().is_some_and(|x|x=="ckpt")).collect::<Vec<_>>();
 let sp=fs::read_dir(&sdir).map_err(|e|e.to_string())?.flatten().map(|e|e.path()).filter(|p|p.extension().is_some_and(|x|x=="pth")).collect::<Vec<_>>();
 let mut chars=Vec::new();
 for band_entry in fs::read_dir(&voices).map_err(|e|e.to_string())?.flatten().filter(|e|e.path().is_dir()){
  let band=band_entry.file_name().to_string_lossy().to_string();
  for ce in fs::read_dir(band_entry.path()).into_iter().flatten().flatten().filter(|e|e.path().is_dir()){
   let folder=ce.file_name().to_string_lossy().to_string();let name=folder.split('（').next().unwrap_or(&folder).to_string();
   let mut refs=WalkDir::new(ce.path()).max_depth(2).into_iter().flatten().filter(|e|e.file_type().is_file()).filter(|e|matches!(e.path().extension().and_then(|x|x.to_str()),Some("mp3"|"wav"))).map(|e|Reference{path:e.path().to_string_lossy().to_string(),text:stem(e.path())}).collect::<Vec<_>>();
   // Reference clips must be 3–10 seconds. Dialogue around 18–28 Japanese
   // characters is the best default before the engine performs exact validation.
   refs.sort_by_key(|r|r.text.chars().count().abs_diff(23));
   if !refs.is_empty(){chars.push(Character{id:format!("{}::{}",band,folder),name:name.clone(),band:band.clone(),variant:folder.split_once('（').map(|x|x.1.trim_end_matches('）').to_string()),gpt_weight:choose_weight(&gp,&name,&folder),sovits_weight:choose_weight(&sp,&name,&folder),references:refs});}
  }
 }
 chars.sort_by(|a,b|a.band.cmp(&b.band).then(a.name.cmp(&b.name)));Ok(Catalog{characters:chars,root:voices.to_string_lossy().into(),engine_root:engine.to_string_lossy().into()})
}

async fn ensure_engine(engine:&Engine,root:&Path)->Result<String,String>{
 let client=http_client()?;let existing_port={*engine.port.lock().map_err(|_|"引擎端口状态锁定失败")?};if let Some(port)=existing_port{let base=format!("http://127.0.0.1:{port}");if client.get(format!("{base}/docs")).send().await.is_ok_and(|r|r.status().is_success()){return Ok(base)}}
 let port=TcpListener::bind("127.0.0.1:0").and_then(|s|s.local_addr().map(|a|a.port())).map_err(|e|format!("无法分配本地推理端口：{e}"))?;
 { let mut guard=engine.child.lock().map_err(|_|"引擎状态锁定失败")?;if let Some(mut old)=guard.take(){let _=old.start_kill();}let log=OpenOptions::new().create(true).truncate(true).write(true).open(app_data_root()?.join("engine.log")).map_err(|e|e.to_string())?;let err=log.try_clone().map_err(|e|e.to_string())?;let bundled_python=root.join(".venv/bin/python-bundled");let python=if bundled_python.is_file(){bundled_python}else{root.join(".venv/bin/python")};let child=Command::new(python).arg("-u").arg("api_v2.py").arg("-a").arg("127.0.0.1").arg("-p").arg(port.to_string()).env("MPLCONFIGDIR","/private/tmp/bangvoice-mpl").env("XDG_CACHE_HOME","/private/tmp/bangvoice-cache").current_dir(root).stdout(Stdio::from(log)).stderr(Stdio::from(err)).spawn().map_err(|e|format!("无法启动内置推理引擎：{e}"))?;*guard=Some(child);}
 *engine.port.lock().map_err(|_|"引擎端口状态锁定失败")?=Some(port);*engine.current.lock().map_err(|_|"模型状态锁定失败")?=None;let base=format!("http://127.0.0.1:{port}");
 for _ in 0..180{sleep(Duration::from_secs(1)).await;if client.get(format!("{base}/docs")).send().await.is_ok_and(|r|r.status().is_success()){return Ok(base)}}Err(format!("推理引擎在端口 {port} 启动超时，请查看 engine/engine.log"))
}
#[tauri::command]
async fn generate_voice(request:GenerateRequest,engine:State<'_,Engine>)->Result<String,String>{generate_voice_core(request,&engine).await}
async fn generate_voice_core(request:GenerateRequest,engine:&Engine)->Result<String,String>{
 let root=project_root()?.join("engine");let base=ensure_engine(&engine,&root).await?;let catalog=load_catalog()?;let c=catalog.characters.into_iter().find(|c|c.id==request.character_id).ok_or("角色不存在")?;let client=http_client()?;
 let g=c.gpt_weight.ok_or("该角色缺少 GPT 权重")?;let s=c.sovits_weight.ok_or("该角色缺少 SoVITS 权重")?;
 let already_loaded=engine.current.lock().map_err(|_|"模型状态锁定失败")?.as_ref()==Some(&(g.clone(),s.clone()));
 if !already_loaded {
  let gr=client.get(format!("{base}/set_gpt_weights")).query(&[("weights_path",&g)]).send().await.map_err(|e|e.to_string())?;if !gr.status().is_success(){return Err(format!("GPT 权重加载失败：{}",gr.text().await.unwrap_or_default()))}
  let sr=client.get(format!("{base}/set_sovits_weights")).query(&[("weights_path",&s)]).send().await.map_err(|e|e.to_string())?;if !sr.status().is_success(){return Err(format!("SoVITS 权重加载失败：{}",sr.text().await.unwrap_or_default()))}
  *engine.current.lock().map_err(|_|"模型状态锁定失败")?=Some((g.clone(),s.clone()));
 }
 let payload=serde_json::json!({"text":request.text,"text_lang":request.text_lang,"ref_audio_path":request.reference_path,"prompt_text":request.prompt_text,"prompt_lang":"ja","speed_factor":request.speed,"temperature":request.temperature,"seed":request.seed,"media_type":"wav","streaming_mode":false,"text_split_method":"cut5","batch_size":1});
 let response=client.post(format!("{base}/tts")).json(&payload).send().await.map_err(|e|e.to_string())?;if !response.status().is_success(){return Err(format!("合成失败：{}",response.text().await.unwrap_or_default()))}let bytes=response.bytes().await.map_err(|e|e.to_string())?;
 let out=output_root()?;let ts=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();let path=out.join(format!("{}_{}.wav",request.character_id.replace([':','/'],"_"),ts));fs::write(&path,bytes).map_err(|e|e.to_string())?;Ok(path.to_string_lossy().into())
}
#[tauri::command]
fn load_history()->Result<Vec<HistoryItem>,String>{let out=output_root()?;let mut items=fs::read_dir(out).into_iter().flatten().flatten().filter_map(|e|{let p=e.path();if p.extension()?.to_str()?!="wav"{return None}let m=e.metadata().ok()?;Some(HistoryItem{path:p.to_string_lossy().into(),name:stem(&p),created:m.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_secs(),size:m.len()})}).collect::<Vec<_>>();items.sort_by(|a,b|b.created.cmp(&a.created));Ok(items)}
#[tauri::command]
fn export_audio(source:String,destination:String)->Result<(),String>{let output=fs::canonicalize(output_root()?).map_err(|e|e.to_string())?;let source=fs::canonicalize(source).map_err(|e|format!("找不到生成音频：{e}"))?;if !source.starts_with(&output)||source.extension().and_then(|x|x.to_str())!=Some("wav"){return Err("只允许导出应用生成的 WAV 文件".into())}fs::copy(source,destination).map_err(|e|format!("保存失败：{e}"))?;Ok(())}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(){let app=tauri::Builder::default().plugin(tauri_plugin_shell::init()).plugin(tauri_plugin_dialog::init()).manage(Engine::default()).invoke_handler(tauri::generate_handler![load_catalog,load_history,generate_voice,export_audio]).build(tauri::generate_context!()).expect("error while building app");app.run(|handle,event|{if matches!(event,tauri::RunEvent::Exit){let engine=handle.state::<Engine>();if let Ok(mut child)=engine.child.lock(){if let Some(mut process)=child.take(){let _=process.start_kill();}};}})}

#[cfg(test)]
mod tests { use super::*; #[test] fn every_voice_folder_has_a_model_pair(){let c=load_catalog().expect("catalog");let missing=c.characters.iter().filter(|x|x.gpt_weight.is_none()||x.sovits_weight.is_none()).map(|x|format!("{}/{}",x.band,x.name)).collect::<Vec<_>>();assert!(missing.is_empty(),"missing model pairs: {missing:?}");}
 #[tokio::test] async fn app_managed_engine_uses_private_port(){let engine=Engine::default();let root=project_root().unwrap().join("engine");let base=ensure_engine(&engine,&root).await.expect("official engine starts");let port=engine.port.lock().unwrap().unwrap();assert!(port>0);assert_eq!(base,format!("http://127.0.0.1:{port}"));assert!(http_client().unwrap().get(format!("{base}/docs")).send().await.unwrap().status().is_success());if let Some(mut child)=engine.child.lock().unwrap().take(){let _=child.start_kill();};}
 #[tokio::test] async fn app_generation_pipeline_creates_wav(){let engine=Engine::default();let root=project_root().unwrap();let ref_path=root.join("角色语音/Mujica/丰川祥子/一体どういうことですの？今日もまた来ていましたわ！.mp3");let request=GenerateRequest{character_id:"Mujica::丰川祥子".into(),text:"ごきげんよう。今日はいい天気ですわね。".into(),text_lang:"ja".into(),reference_path:ref_path.to_string_lossy().into(),prompt_text:"一体どういうことですの？今日もまた来ていましたわ！".into(),speed:1.0,temperature:1.0,seed:42};let output=generate_voice_core(request,&engine).await.expect("app pipeline generates audio");let data=fs::read(output).unwrap();assert!(data.len()>44);assert_eq!(&data[0..4],b"RIFF");if let Some(mut child)=engine.child.lock().unwrap().take(){let _=child.start_kill();};}
 #[test] fn native_audio_export_copies_wav(){let source=output_root().unwrap().join("export-command-test.wav");fs::write(&source,b"RIFFtest").unwrap();let destination=std::env::temp_dir().join("bangvoice-export-command-test.wav");export_audio(source.to_string_lossy().into(),destination.to_string_lossy().into()).unwrap();assert_eq!(fs::read(&destination).unwrap(),b"RIFFtest");let _=fs::remove_file(source);let _=fs::remove_file(destination);}
}
