FROM tr_logic_llc/base

# Копирование файлов проекта
COPY ./app .

RUN cargo install --path .

# Уведомление о порте, который будет прослушивать работающее приложение
EXPOSE 8000

# Запуск проекта
CMD ["tr_logic_llc"]
