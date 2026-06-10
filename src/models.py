from __future__ import annotations

import json
from dataclasses import dataclass, field, asdict
from typing import Optional


@dataclass
class CharacterConfig:
    name: str
    gltf_ref: str


@dataclass
class Manifest:
    prompt: str
    characters: list[CharacterConfig]
    duration_seconds: Optional[float] = None

    @classmethod
    def from_dict(cls, data: dict) -> Manifest:
        chars = []
        for c in data.get("characters", []):
            chars.append(CharacterConfig(
                name=c["name"],
                gltf_ref=c["gltf_ref"],
            ))
        return cls(
            prompt=data["prompt"],
            characters=chars,
            duration_seconds=data.get("duration_seconds"),
        )

    def to_dict(self) -> dict:
        return asdict(self)


@dataclass
class IterationState:
    iteration_number: int
    scene_gltf: dict
    critique: Optional[str] = None
    convergence_decision: Optional[str] = None
    errors: list[str] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "iteration_number": self.iteration_number,
            "critique": self.critique,
            "convergence_decision": self.convergence_decision,
            "errors": self.errors,
        }


@dataclass
class SceneState:
    manifest: Manifest
    current_scene_gltf: dict
    iteration_history: list[IterationState]
    iteration_count: int
    converged: bool = False


def load_manifest_from_string(text: str) -> Manifest:
    return Manifest.from_dict(json.loads(text))
