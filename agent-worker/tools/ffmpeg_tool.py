# Required imports for FFmpeg tool functionality
import asyncio
import subprocess
from typing import Optional
from langchain.callbacks.manager import (
    CallbackManagerForToolRun,
    AsyncCallbackManagerForToolRun,
)
from langchain.tools import BaseTool
from langchain.chains import LLMChain
from langchain.llms import OpenAI
from langchain_anthropic import ChatAnthropic
from langchain.prompts import PromptTemplate


class FfmpegTool(BaseTool):
    """
    A LangChain tool that provides FFmpeg audio manipulation capabilities through natural language.
    This tool uses OpenAI's LLM to convert natural language requests into valid FFmpeg commands.

    Features:
    - Audio file clipping
    - Format conversion
    - Quality/bitrate adjustment
    - Audio extraction from video
    """

    name: str = "FfmpegTool"
    description: str = """Tool for audio file manipulation using FFmpeg.
    Can perform operations like:
    - Clipping audio files to specific durations
    - Converting between audio formats
    - Adjusting audio quality and bitrate
    - Extracting audio from video
    Input should be a clear description of the desired audio operation."""

    def _run(
        self,
        query: str,
        run_manager: Optional[CallbackManagerForToolRun] = None
    ) -> str:
        """
        Synchronous execution of FFmpeg commands.

        Flow:
        1. Takes natural language query
        2. Converts to FFmpeg command via LLM
        3. Executes command and returns output

        Args:
            query: Natural language description of desired audio operation
            run_manager: Optional callback manager for monitoring execution

        Returns:
            Command output or error message as string
        """
        # Template for LLM to convert natural language to FFmpeg command
        prompt = f"""Question or Query: {query}

  Generate a valid ffmpeg command to accomplish this task.
    Only return the command itself, no explanations or additional text.
    Use relative paths starting with ./samples/ for input and output files.
        """

        # Set up LLM chain to generate FFmpeg command
        llm = ChatAnthropic(
            model_name="claude-3-sonnet-20240229",
            timeout=60,
            stop=None,
        )
        messages = [
            (
                "system",
                "You're an expert audio engineer specializing in FFmpeg operations.",
            ),
            ("human", prompt),
        ]
        content = llm.invoke(messages).content
        command: str = str(content)

        # Parse and validate the generated command
        cmd_parts = [part.strip() for part in command.split() if part.strip()]

        if not cmd_parts or cmd_parts[0] != "ffmpeg":
            return "Invalid command generated"

        # Prepare command with standard flags for consistent execution
        if cmd_parts[0] == "ffmpeg":
            cmd_parts = cmd_parts[1:]

        # Add standard flags:
        # -loglevel error: Only show errors
        # -y: Overwrite output files without asking
        full_cmd = ["ffmpeg", "-loglevel", "error", "-y"] + cmd_parts

        print(f'\n\nrunning command: {" ".join(full_cmd)}')
        output = subprocess.run(full_cmd, capture_output=True),

        # Handle tuple output from subprocess.run()
        if type(output) == tuple:
            output = output[0]

        # Process command output
        stdout = output.stdout
        stderr = output.stderr
        returncode = output.returncode
        print(f'[ffmpeg exited with {returncode}]')
        if stdout:
            print(f'\n{stdout.decode()}')
        if stderr:
            return f'[there was an error]\n{stderr.decode()}'

        return stdout.decode()

    async def _arun(
            self,
            query: str,
            run_manager: Optional[AsyncCallbackManagerForToolRun] = None
        ) -> str:
            """
            Asynchronous execution of FFmpeg commands.

            Provides non-blocking execution for long-running FFmpeg operations.

            Args:
                query: Natural language description of desired audio operation
                run_manager: Optional async callback manager for monitoring execution

            Returns:
                Status message indicating operation completion
            """

            async def async_run(cmd):
                """
                Inner async function to handle subprocess execution
                Captures both stdout and stderr for proper error handling
                """
                proc = await asyncio.create_subprocess_shell(
                    cmd,
                    stdout=asyncio.subprocess.PIPE,
                    stderr=asyncio.subprocess.PIPE,
                )

                stdout, stderr = await proc.communicate()

                print(f'[{cmd!r} exited with {proc.returncode}]')
                if stdout:
                    print(f'[stdout]\n{stdout.decode()}')
                if stderr:
                    print(f'[stderr]\n{stderr.decode()}')
                    raise Exception(f'[stderr]\n{stderr.decode()}')

                return stdout.decode()

            # Template for LLM to convert natural language to FFmpeg command
            prompt = f"""Question or Query: {query}

        Generate a valid ffmpeg command to accomplish this task.
        Only return the command itself, no explanations or additional text.
        Use relative paths starting with ./samples/ for input and output files.
            """

            # Set up LLM chain to generate FFmpeg command
            llm = ChatAnthropic(
                model_name="claude-3-sonnet-20240229",
                timeout=60,
                stop=None,
            )
            messages = [
                (
                    "system",
                    "You're an expert audio engineer specializing in FFmpeg operations.",
                ),
                ("human", prompt),
            ]
            content = llm.invoke(messages).content
            command: str = str(content)

            # Parse and validate the generated command
            cmd_parts = [part.strip() for part in command.split() if part.strip()]

            if not cmd_parts or cmd_parts[0] != "ffmpeg":
                return "Invalid command generated"

            # Prepare command with standard flags for consistent execution
            if cmd_parts[0] == "ffmpeg":
                cmd_parts = cmd_parts[1:]

            # Add standard flags:
            # -loglevel error: Only show errors
            # -y: Overwrite output files without asking
            full_cmd = ["ffmpeg", "-loglevel", "error", "-y"] + cmd_parts

            return await async_run(full_cmd)
