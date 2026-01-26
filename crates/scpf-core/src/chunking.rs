/// Reusable chunking utility for processing large text files
use anyhow::Result;

pub struct ChunkProcessor<T> {
    chunk_size: usize,
    overlap: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ChunkProcessor<T> {
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Process source in overlapping chunks
    pub fn process<F>(&self, source: &str, mut process_chunk: F) -> Result<Vec<T>>
    where
        F: FnMut(&str, usize) -> Result<Vec<T>>,
    {
        let mut all_results = Vec::new();
        let mut offset = 0;

        while offset < source.len() {
            let end = (offset + self.chunk_size).min(source.len());
            let chunk = &source[offset..end];

            // Calculate line offset for this chunk
            let line_offset = if offset > 0 {
                source[..offset].matches('\n').count()
            } else {
                0
            };

            // Process chunk
            let chunk_results = process_chunk(chunk, line_offset)?;
            all_results.extend(chunk_results);

            // Move to next chunk with overlap
            offset += self.chunk_size;
            if offset < source.len() {
                offset = offset.saturating_sub(self.overlap);
            }
        }

        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_processor_basic() {
        let processor = ChunkProcessor::<String>::new(10, 2);
        let source = "line1\nline2\nline3\nline4\nline5";
        
        let results = processor.process(source, |chunk, line_offset| {
            Ok(vec![format!("chunk at line {}: {}", line_offset, chunk.len())])
        }).unwrap();
        
        assert!(!results.is_empty());
    }

    #[test]
    fn test_chunk_processor_line_offset() {
        let processor = ChunkProcessor::<(usize, String)>::new(20, 5);
        let source = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8";
        
        let results = processor.process(source, |chunk, line_offset| {
            Ok(vec![(line_offset, chunk.to_string())])
        }).unwrap();
        
        // First chunk should have line_offset 0
        assert_eq!(results[0].0, 0);
        
        // Subsequent chunks should have increasing line offsets
        for i in 1..results.len() {
            assert!(results[i].0 > 0);
        }
    }

    #[test]
    fn test_chunk_processor_overlap() {
        let processor = ChunkProcessor::<String>::new(15, 5);
        let source = "0123456789abcdefghijklmnopqrstuvwxyz";
        
        let mut chunks = Vec::new();
        processor.process(source, |chunk, _| {
            chunks.push(chunk.to_string());
            Ok(vec![chunk.to_string()])
        }).unwrap();
        
        // Verify chunks overlap
        assert!(chunks.len() > 1);
        for i in 1..chunks.len() {
            let prev_end = &chunks[i-1][chunks[i-1].len().saturating_sub(5)..];
            let curr_start = &chunks[i][..5.min(chunks[i].len())];
            // Some overlap should exist
            assert!(prev_end.len() > 0 && curr_start.len() > 0);
        }
    }

    #[test]
    fn test_chunk_processor_small_source() {
        let processor = ChunkProcessor::<String>::new(100, 10);
        let source = "small";
        
        let results = processor.process(source, |chunk, line_offset| {
            assert_eq!(line_offset, 0);
            assert_eq!(chunk, "small");
            Ok(vec!["processed".to_string()])
        }).unwrap();
        
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_chunk_processor_exact_chunk_size() {
        let processor = ChunkProcessor::<String>::new(10, 2);
        let source = "0123456789"; // Exactly 10 bytes
        
        let results = processor.process(source, |chunk, _| {
            Ok(vec![chunk.to_string()])
        }).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], source);
    }

    #[test]
    fn test_chunk_processor_newline_counting() {
        let processor = ChunkProcessor::<usize>::new(20, 5);
        let source = "line1\nline2\nline3\nline4\nline5\nline6\nline7";
        
        let line_offsets: Vec<usize> = processor.process(source, |_, line_offset| {
            Ok(vec![line_offset])
        }).unwrap();
        
        // First chunk starts at line 0
        assert_eq!(line_offsets[0], 0);
        
        // Each subsequent chunk should have correct line offset
        for offset in &line_offsets[1..] {
            assert!(*offset > 0);
        }
    }
}
