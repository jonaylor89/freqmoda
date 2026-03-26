export interface AudioSample {
  streamingKey: string;
  title: string;
  duration: number;
  fileType: string;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  audioUrl?: string;
}
