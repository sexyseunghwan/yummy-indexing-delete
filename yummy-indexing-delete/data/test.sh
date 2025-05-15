#!/bin/bash

# 실행파일 경로 설정
EXEC_PATH="/home/seunghwan/Documents/yummy-indexing-delete/yummy-indexing-delete/yummy-indexing-delete"

# 실행파일이 존재하고 실행 가능하지 않다면 로그 남기고 종료
if [ ! -x "$EXEC_PATH" ]; then
    logger -t run_my_program "ERROR: 실행 파일이 없거나 실행 권한 없음: $EXEC_PATH"
    exit 1
fi

# 실행 (에러 발생 시 로그 남김)
"$EXEC_PATH" || logger -t run_my_program "ERROR: 실행 중 오류 발생: $EXEC_PATH"