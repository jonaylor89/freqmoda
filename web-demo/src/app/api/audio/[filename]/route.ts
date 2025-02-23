import { NextResponse } from 'next/server';
import { promises as fs } from 'fs';
import path from 'path';

export async function GET(
  request: Request,
  { params }: { params: { filename: string } }
) {
  try {
    const audioPath = path.join(process.cwd(), 'src', 'data', 'audio', params.filename);
    
    const exists = await fs.access(audioPath).then(() => true).catch(() => false);
    if (!exists) {
      return new NextResponse('File not found', { status: 404 });
    }

    // Handle range requests for better streaming
    const range = request.headers.get('range');
    const stat = await fs.stat(audioPath);
    
    if (range) {
      const parts = range.replace(/bytes=/, '').split('-');
      const start = parseInt(parts[0], 10);
      const end = parts[1] ? parseInt(parts[1], 10) : stat.size - 1;
      const chunksize = (end - start) + 1;
      const file = await fs.open(audioPath);
      const buffer = Buffer.alloc(chunksize);
      await file.read(buffer, 0, chunksize, start);
      await file.close();

      return new Response(buffer, {
        status: 206,
        headers: {
          'Content-Range': `bytes ${start}-${end}/${stat.size}`,
          'Accept-Ranges': 'bytes',
          'Content-Length': chunksize.toString(),
          'Content-Type': 'audio/mpeg',
        },
      });
    }

    // Full file response
    const file = await fs.readFile(audioPath);
    return new Response(file, {
      headers: {
        'Content-Type': 'audio/mpeg',
        'Content-Length': stat.size.toString(),
        'Accept-Ranges': 'bytes',
      },
    });
  } catch (error) {
    console.error('Error:', error);
    return new NextResponse('Server error', { status: 500 });
  }
} 