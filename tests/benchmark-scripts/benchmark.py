cycles = 1000000
count = 0
acum = 0.0
result_text = ""
status = True

i = 0
while i < cycles:
    
    count = count + 1
    
    acum = acum + 0.5
    
    if i % 2 == 0:
        status = True
    else:
        status = False
    
    if status == True:
        result_text = "par"
    else:
        result_text = "impar"
    
    i = i + 1

print(count)
print(acum)
print(result_text)
print(status)
