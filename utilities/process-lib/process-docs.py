from langchain_text_splitters import MarkdownTextSplitter
from langchain_community.vectorstores import Chroma
from langchain_community.embeddings import HuggingFaceEmbeddings
from langchain_community.llms import HuggingFacePipeline
from langchain.chains import RetrievalQA
from langchain.prompts import PromptTemplate
import sys
import torch
import os

def main():
    
    device = "cpu"
    if torch.cuda.is_available():
        device = "cuda"
        print(f"NVIDIA GPU detected: {torch.cuda.get_device_name(0)}")
        print(f"Using CUDA for embeddings generation")
    elif hasattr(torch.backends, 'mps') and torch.backends.mps.is_available():
        device = "mps"
        print(f"Apple Silicon GPU detected")
        print(f"Using MPS for embeddings generation")
    else:
        print("No GPU detected. Using CPU for embeddings generation.")
    

    with open("api_documentation.md") as f:
        markdown_text = f.read()
    

    from transformers import AutoModelForSeq2SeqLM, AutoTokenizer, pipeline

    print("Loading LLM model (this may take a moment)...")
    
    # Use a smaller model that works well on CPU
    model_id = "google/flan-t5-large"
    
    tokenizer = AutoTokenizer.from_pretrained(model_id)
    model = AutoModelForSeq2SeqLM.from_pretrained(
        model_id, 
        device_map="auto",
        torch_dtype=torch.float16 if device != "cpu" else torch.float32
    )
    
    # Calculate appropriate chunk size based on tokenizer
    # For flan-t5-large, context window is ~512 tokens
    # Reserve ~200 tokens for the question and answer
    # This leaves ~300 tokens for context
    # Assuming average of 4 chars per token, aim for ~1200 chars per chunk
    chunk_size = 1200
    chunk_overlap = 300
    
    print(f"Using chunk size of {chunk_size} characters with {chunk_overlap} overlap")
    
    text_splitter = MarkdownTextSplitter(chunk_size=chunk_size, chunk_overlap=chunk_overlap)
    docs = text_splitter.create_documents([markdown_text])
    
    print(f"Split documentation into {len(docs)} chunks")
    
    # Create vector embeddings using a local HuggingFace model with GPU if available
    embeddings = HuggingFaceEmbeddings(
        model_name="all-MiniLM-L6-v2",
        model_kwargs={"device": device}
    )
    db = Chroma.from_documents(docs, embeddings)
    
    # Increase max_length for the model to handle longer inputs and outputs
    pipe = pipeline(
        "text2text-generation",
        model=model,
        tokenizer=tokenizer,
        max_length=1024,  # Increased from 512 to 1024 for longer outputs
        temperature=0.7,  # Add some temperature for more detailed responses
        num_return_sequences=1,
    )
    
    llm = HuggingFacePipeline(pipeline=pipe)
    
    # Create a custom prompt template with shorter context
    prompt_template = """
    Answer the following question about the process-Lib API using the provided context.
    If the information isn't in the context, say you don't know.
    
    Context:
    {context}
    
    Question: {question}
    
    Answer:
    """
    
    PROMPT = PromptTemplate(
        template=prompt_template,
        input_variables=["context", "question"]
    )
    
    # Create a retrieval chain with appropriate context handling
    qa_chain = RetrievalQA.from_chain_type(
        llm=llm,
        chain_type="stuff",  # "stuff" puts all retrieved docs into context at once
        retriever=db.as_retriever(
            search_kwargs={
                "k": 3,  # Increased from 2 to 3 documents
                "score_threshold": 0.5,  # Only include relevant documents
            }
        ),
        chain_type_kwargs={
            "prompt": PROMPT,
            "document_separator": "\n\n",  # Shorter separator
        },
        return_source_documents=True
    )
    
    # For file handling query
    if len(sys.argv) <= 1:
        query = "file handling operations VFS filesystem read write open close"
        print(f"Using default file handling query: {query}\n")
    else:
        query = " ".join(sys.argv[1:])
        print(f"Query: {query}\n")
    
    # Run the chain
    try:
        result = qa_chain.invoke({"query": query})  # Using invoke instead of __call__
        
        # Print the answer
        print("\n" + "="*80)
        print("LLM GENERATED ANSWER:")
        print("="*80 + "\n")
        print(result["result"])
        print("\n" + "-"*80 + "\n")
        
        # Print source documents
        print("SOURCE DOCUMENTS:")
        for i, doc in enumerate(result["source_documents"]):
            print(f"\n{'='*80}\n=== SOURCE {i+1} ===\n{'='*80}\n")
            print(doc.page_content[:1000] + "..." if len(doc.page_content) > 1000 else doc.page_content)
            print(f"\n{'-'*80}\n")
            
    except Exception as e:
        print(f"Error: {e}")
        print("\nFalling back to simple search without LLM...")
        
        # Fallback to simple search
        docs = db.similarity_search(query, k=1)
        
        print("\nRelevant documentation sections:")
        for i, doc in enumerate(docs):
            print(f"\n{'='*80}\n=== RESULT {i+1} ===\n{'='*80}\n")
            print(doc.page_content)
            print(f"\n{'-'*80}\n")

if __name__ == "__main__":
    # Configure environment variable to avoid tokenizer warnings
    os.environ["TOKENIZERS_PARALLELISM"] = "false"
    main()