import { Command } from '../types.js';
export interface InsertAssetArgs {
    path: string;
    caption?: string;
}
export declare const InsertAssetCommand: Command<InsertAssetArgs>;
