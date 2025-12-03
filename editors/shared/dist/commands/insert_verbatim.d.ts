import { Command } from '../types.js';
export interface InsertVerbatimArgs {
    path: string;
    content: string;
    language: string;
}
export declare const InsertVerbatimCommand: Command<InsertVerbatimArgs>;
