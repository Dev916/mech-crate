.PHONY: ecs _ecs

# Connect to a running ECS task. (make s=[service]) (ex. make ecs s=campus-lms)
ecs:
	@$(MAKE) _ecs service=$(call get_service)

# Connect to a running ECS task. (make s=[service]) (ex. make ecs s=campus-lms)
_ecs:
	@./scripts/aws/ecs.sh $(service)