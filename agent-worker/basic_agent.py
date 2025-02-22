# Import relevant functionality
from langchain_anthropic import ChatAnthropic
from langchain_community.tools.tavily_search import TavilySearchResults
from langchain_core.messages import HumanMessage
from langgraph.checkpoint.memory import MemorySaver
from langgraph.prebuilt import create_react_agent
from dotenv import load_dotenv
load_dotenv()

# Saves the history in a vector store to be query-able
memory = MemorySaver()

# REQUIRES A CLAUDE API KEY
model = ChatAnthropic(
    model_name="claude-3-sonnet-20240229",
    timeout=60,  # Maximum time to wait for model response
    stop=None    # No custom stop tokens
)

# USING TAVILY FOR SEARCH
search = TavilySearchResults(max_results=2)
tools = [search]

# ReAct Executioner
agent_executor = create_react_agent(model, tools, checkpointer=memory)

# Use the agent
config = {"configurable": {"thread_id": "abc123"}}
for chunk in agent_executor.stream(
    {"messages": [HumanMessage(content="hi im bob! and i live in sf")]}, config
):
    try:
        messages = chunk["agent"]["messages"]
        for message in messages:
            print(message.content)
        print("----")
    except:
        continue


for chunk in agent_executor.stream(
    {"messages": [HumanMessage(content="whats the weather where I live?")]}, config
):

    try:
        messages = chunk["agent"]["messages"]
        for message in messages:
            print(message.content)
        print("----")
    except:
        continue
