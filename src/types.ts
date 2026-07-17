export type Character = { id:string; name:string; band:string; variant?:string; gptWeight?:string; sovitsWeight?:string; references:{path:string;text:string}[] };
export type Catalog = { characters:Character[]; root:string; engineRoot:string };
export type GenerateRequest = { characterId:string; text:string; textLang:string; referencePath:string; promptText:string; speed:number; temperature:number; seed:number };
export type HistoryItem = { path:string; name:string; created:number; size:number };
