###############################################################################
# FFmpeg Audio Processing Agent Tutorial
#
# This script demonstrates how to create an AI agent specialized in audio processing
# using FFmpeg. It combines LangChain, Anthropic's Claude, and custom tools to
# create an interactive audio engineering assistant.
###############################################################################

# Import required libraries
from dotenv import load_dotenv
from langchain_anthropic import ChatAnthropic
from langchain_community.tools.tavily_search import TavilySearchResults
from langchain_core.messages import HumanMessage
from langgraph.checkpoint.memory import MemorySaver
from langgraph.prebuilt import create_react_agent

from tools.ffmpeg_tool import FfmpegTool

# Load environment variables (e.g. API keys)
load_dotenv()


def main():
    ###########################################################################
    # Agent Setup
    ###########################################################################

    # Initialize memory to maintain conversation state
    memory = MemorySaver()

    # Configure the Claude 3 Sonnet model as our main LLM
    model = ChatAnthropic(
        model_name="claude-3-sonnet-20240229",
        timeout=60,  # Maximum time to wait for model response
        stop=None    # No custom stop tokens
    )

    # Initialize tools that the agent can use:
    # - TavilySearchResults: For web searches (limited to 2 results for conciseness)
    # - FfmpegTool: Custom tool for audio manipulation
    search = TavilySearchResults(max_results=2)
    tools = [search, FfmpegTool()]

    # Define the agent's core behavior and expertise through a prompt
    prompt = """You are an expert audio engineer specializing in FFmpeg operations.
    When asked to perform audio operations:
    1. Use the FfmpegTool for any audio file manipulations
    2. Be precise with file paths, always using ./samples/ directory
    3. Verify the command's success and report any errors
    4. Only use valid FFmpeg parameters and syntax

    Remember to check the output of commands and handle any errors appropriately."""

    # Create the reactive agent with our model, tools, and configuration
    agent_executor = create_react_agent(model, tools, checkpointer=memory, state_modifier=prompt)

    ###########################################################################
    # Agent Interaction Examples
    ###########################################################################

    # Configuration for maintaining conversation thread
    config = {"configurable": {"thread_id": "abc123"}}

    # Demonstrate the agent's ability to handle an FFmpeg command request
    # This example asks the agent to clip and convert an audio file
    usr_cmds = [
        # "clip the audio file located at ./samples/helicopter.wav to be 5 seconds, save it as an mp3 file",
        # "speed up the audio file located at ./samples/ratatat.mp3 by 50% and save it as a wav file",
        # "add some echo the audio file located at ./samples/ratatat.mp3 and save it as ./samples/ratatat_echo.mp3",
        # "convert the audio file located at ./samples/ratatat.mp3 to be in flac format",
    ]
    for usr_cmd in usr_cmds:
        for chunk in agent_executor.stream(
            {"messages": [HumanMessage(content=usr_cmd)]},
            config,
        ):
            print(chunk)
            print("----")


# Standard Python idiom for running the main function
if __name__ == '__main__':
    main()
