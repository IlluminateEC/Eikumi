import asyncio
import json
import os
import sys
import logging
from pathlib import Path
import traceback
from typing import TypedDict, cast

import discord
from discord.ext import commands
from discord import Intents
import rich
import rich.logging
import rich.traceback


MODULE_DIRECTORY = Path(__file__).parent.parent / "cogs"
CONFIG_PATH = Path("config.json")

type Context = commands.Context[commands.Bot]
ConfigGuild = TypedDict(
    "ConfigGuild",
    {
        "transparency_channel": int,
        "membership_channel": int,
    },
)
Config = TypedDict(
    "Config",
    {
        "guilds": dict[str, ConfigGuild],
    },
)


class Eikumi:
    def __init__(self) -> None:
        self.bot = commands.Bot(command_prefix=">>", intents=Intents.all())
        self.bot._eikumi = self  # type: ignore

        if not CONFIG_PATH.exists():
            logging.critical("`config.json` does not exist")
            sys.exit(1)

        try:
            self.config: Config = json.load(CONFIG_PATH.open())

        except json.JSONDecodeError as error:
            logging.critical("Could not parse `config.json`", exc_info=error)
            sys.exit(1)

    def find_modules(self) -> list[str]:
        files = list(MODULE_DIRECTORY.glob("*.py"))

        return [
            str(module.with_suffix("").relative_to(MODULE_DIRECTORY))
            .replace("./", "")
            .replace("/", ".")
            for module in files
        ]

    def module_is_loaded(self, module: str) -> bool:
        return self.bot.extensions.get(f"cogs.{module}") is not None

    async def load_module(self, module: str) -> tuple[str, Exception | None]:
        try:
            if self.module_is_loaded(module):
                await self.bot.reload_extension(f"cogs.{module}")

            else:
                await self.bot.load_extension(f"cogs.{module}")

        except Exception as error:
            logging.error(f"failed to load {module}", exc_info=error)
            # todo: proper error reporting

            return (module, error)

        return (module, None)

    async def load_modules(
        self, modules: list[str]
    ) -> list[tuple[str, Exception | None]]:
        errors = [await self.load_module(module) for module in modules]

        # for module, error in errors:
        #     if error is None:
        #         continue

        return errors

    def run(self) -> None:
        if not (token := os.environ.get("DISCORD_TOKEN")):
            logging.fatal("DISCORD_TOKEN environment variable is not set. Exiting.")
            sys.exit(1)

        @self.bot.listen()
        async def on_ready() -> None:
            logging.info("(re)connected to the Discord API")
            await self.load_modules(self.find_modules())

        @self.bot.command("load")
        @commands.is_owner()
        async def load_extension(ctx: Context, module: str) -> None:
            if (result := await self.load_module(module)) is not None:
                await ctx.send(
                    embed=discord.Embed(
                        title=f"Failed to load `{module}`",
                        description="```py\n{}\n```".format(
                            "".join(traceback.format_exception(result[1]))
                        ),
                        color=0x9C0B21,
                    )
                )

            else:
                await ctx.send(f":white_check_mark: Successfully loaded {module}")

        @self.bot.command("unload")
        @commands.is_owner()
        async def unload_extension(ctx: Context, module: str) -> None:
            if not self.module_is_loaded(module):
                await ctx.send(f":x: {module} is not loaded")
                return

            await self.bot.unload_extension(f"cogs.{module}")
            await ctx.send(f":white_check_mark: Unloaded {module}")

        @self.bot.command("extensions")
        @commands.is_owner()
        async def list_extensions(ctx: Context) -> None:
            module_is_loaded = {
                module: self.module_is_loaded(module) for module in self.find_modules()
            }
            embed = discord.Embed(title="Extension Status", color=0x2ECC71)

            for module, is_loaded in module_is_loaded.items():
                embed.add_field(
                    name=module, value=":white_check_mark:" if is_loaded else ":x:"
                )

            await ctx.send(embed=embed)

        @self.bot.command("reloadall")
        @commands.is_owner()
        async def reload_extensions(ctx: Context) -> None:
            errors = await self.load_modules(self.find_modules())

            if errors:
                failed_modules = [
                    module for (module, error) in errors if error is not None
                ]

                await ctx.send(
                    ":white_check_mark: Failed to reload {failed_extensions.join(', ')}"
                )
                return

            await ctx.send(":white_check_mark: Successfully reloaded all extensions")

        # todo: error reporting

        self.bot.run(
            token,
            log_handler=rich.logging.RichHandler(highlighter=None, markup=True),
            log_formatter=logging.Formatter("[magenta]%(name)s[/magenta] %(message)s"),
        )
