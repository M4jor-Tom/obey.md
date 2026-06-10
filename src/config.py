import sys
from loguru import logger

MAX_ITERATIONS = 10
HARD_MAX_ITERATIONS = 100
MAX_SCENE_DURATION = 60
MAX_KEYFRAMES_PER_CHANNEL = 10000
MAX_RETRIES = 3
CONTEXT_BUDGET_TOKENS = 128_000

LOG_FORMAT = "{time:HH:mm:ss.SSS} | {level:<7} | {name}:{function}:{line} | {message}"


def configure_logging(level: str = "INFO", log_file: str | None = None) -> None:
    logger.remove()
    logger.add(sys.stderr, format=LOG_FORMAT, level=level, colorize=True)
    if log_file:
        logger.add(log_file, format=LOG_FORMAT, level="DEBUG")
