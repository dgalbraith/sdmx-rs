<details>
<summary>XSD contract: <code>GroupType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="GroupType">
		<xs:annotation>
			<xs:documentation>GroupType describes the structure of a group descriptor in a data structure definition. A group may consist of a of partial key, or collection of distinct cube regions or key sets to which attributes may be attached. The purpose of a group is to specify attributes values which have the same value based on some common dimensionality. All groups declared in the data structure must be unique - that is, you may not have duplicate partial keys. All groups must be given unique identifiers.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="GroupBaseType">
				<xs:choice>
					<xs:element ref="GroupDimension" maxOccurs="unbounded"/>
				</xs:choice>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
