const prompt=require("prompt-sync")();
//problem 1
// let age =prompt("enter your age ");
// if(age >10 && age <20){
//     console.log("your age lies between the age limit ")
// }else{
//     console.log("your age does not lies between ")
// }
//problem 2
// let age = prompt("enter your age ")
// switch (age){
//     case '11':
//         console.log("your age is 11")
//         break
//     case '12':
//         console.log("your age is 12")
//         break
//     case '13':
//         console.log("your age is 13")
//         break
//     case '14':
//         console.log("your age is 14")
//         break
//     case '15':
//         console.log("your age is 15")
//         break
//     case '16':
//         console.log("your age is 16")
//         break   
//     default:
//         console.log("enter age between 11 and 16") 
// }


//problem 3
// let num =prompt("enter the number  ");
// num=Number.parseInt(num)
// if(num%2==0 && num%3==0){
//     console.log("number is divisible by 2 and 3")
// }else{
//     console.log("not divisible by 2 and 3")
// }

//problem 4
// let num =prompt("enter the number  ");
// num=Number.parseInt(num)
// if(num%2==0 || num%3==0){
//     console.log("number is either divisible by 2 or 3")
// }else{
//     console.log("not divisible by 2 and 3")
// }

//problem 5
let age =prompt("enter your age ")
let a=age >18?"you can drive ":"you cannot drive "
console.log(a)
